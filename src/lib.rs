//! # rVRRPd library
#![allow(non_snake_case)]

// libc
extern crate libc;
use libc::{c_void, recvfrom, sockaddr, sockaddr_ll, socket, socklen_t, AF_PACKET, SOCK_RAW};

// foreign-types
#[macro_use]
extern crate foreign_types;

// itertools
extern crate itertools;
use itertools::Itertools;

// serde
extern crate serde;
#[macro_use]
extern crate serde_derive;

// deamonize
extern crate daemonize;
use daemonize::Daemonize;

// chrono
extern crate chrono;

// generic constants
mod constants;
use constants::*;

// VRRP data structure
mod packets;
use packets::VRRPpkt;

// operating systems support
mod os;
use os::linux::drivers::NetDrivers;
use os::linux::libc::IfAddrs;
use os::linux::netdev::PflagOp;

// finite state machine
mod fsm;

// checksums
mod checksums;

// address resolution protocol
mod arp;

// timers
mod timers;

// channels and threads
use std::sync::mpsc;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};

// threads pool
mod threads;
use threads::ThreadPool;

// config
mod config;
use config::decode_config;

// protocols
mod protocols;
use protocols::{Protocols, Static};

// authentication
mod auth;
use auth::gen_auth_data;

// debug
mod debug;
use debug::{print_debug, Verbose};

// std
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::mem;
use std::net::IpAddr::V4;
use std::ptr;
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};

/// Library Config Structure
///
/// Includes library configuration parameters
pub struct Config {
    iface: Option<String>,
    mode: u8,
    conf: Option<String>,
    debug: Option<u8>,
}

// Config Implementation
impl Config {
    // new() method
    pub fn new(iface: Option<String>, mode: u8, conf: Option<String>, debug: Option<u8>) -> Config {
        Config {
            iface,
            mode,
            conf,
            debug,
        }
    }
    // iface() getter
    pub fn iface(&self) -> String {
        match &self.iface {
            Some(s) => s.clone(),
            None => String::new(),
        }
    }
    // mode() getter
    pub fn mode(&self) -> &u8 {
        &self.mode
    }
    // conf() getter
    pub fn conf(&self) -> String {
        match &self.conf {
            Some(s) => s.clone(),
            // default configuration file path
            None => RVRRPD_DFLT_CFG_FILE.to_string(),
        }
    }
    // debug() getter
    pub fn debug(&self) -> Option<u8> {
        self.debug
    }
}

/// Virtual Router Structure
#[derive(Debug)]
pub struct VirtualRouter {
    states: fsm::States,
    parameters: fsm::Parameters,
    timers: fsm::Timers,
    flags: fsm::Flags,
}

// VirtualRouter Type Implementation
impl VirtualRouter {
    // new() method
    // create new VirtualRouter
    fn new(
        vrid: u8,
        ifname: String,
        prio: u8,
        vip: [u8; 4],
        advertint: u8,
        preempt: bool,
        rfc3768: bool,
        auth_type: u8,
        auth_secret: Option<String>,
        protocols: Arc<Mutex<Protocols>>,
        debug: &Verbose,
        netdrv: NetDrivers,
    ) -> io::Result<VirtualRouter> {
        // get ifindex from interface name
        let ifindex = match os::linux::libc::c_ifnametoindex(&ifname) {
            Ok(i) => i as i32,
            Err(e) => return Err(e),
        };

        // create new IPv4 addresses vector
        let mut v4addrs = Vec::new();

        // create new IPv4 netmasks vecto
        let mut v4masks = Vec::new();

        // get list of all ip address per interfaces
        let addrlist = IfAddrs::get().unwrap();
        // create a vector of tuples (ifname: &str, ipaddr: IpAddr, netmask: IpAddr)
        let addrlist = addrlist
            .iter()
            .map(|a| (a.name(), a.addr(), a.netmask()))
            .collect::<Vec<_>>();

        // for every tuples in addrlist:
        // if the key matches the vr's interface, push the converted IPv4 address
        // into the v4addrs vector.
        for t in addrlist {
            // take the address and netmask of the matching vr's interface
            if t.0.to_lowercase() == ifname {
                if let Some(V4(ip)) = t.1 {
                    v4addrs.push(ip.octets());
                    if let Some(V4(netmask)) = t.2 {
                        v4masks.push(netmask.octets())
                    }
                }
            }
        }
        // make sure there is a least one ip/mask pair, otherwise return an error
        if v4addrs.is_empty() || v4masks.is_empty() {
            println!(
                "error(vr): at least one IPv4 address must be available on interface {}",
                ifname
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "no ip address configured on vr's interface",
            ));
        }

        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_MAIN,
            format!(
                "creating new virtal-router, vrid {} on interface {}, ipaddrs {:?}",
                vrid, ifname, v4addrs
            ),
        );

        // verify authentication settings
        match auth_type {
            // if authentication types require a secret
            1 | 250 => {
                if auth_secret.is_none() {
                    print_debug(
                        debug,
                        DEBUG_LEVEL_MEDIUM,
                        DEBUG_SRC_VR,
                        format!("no authentication secret configured"),
                    );
                }
            }
            _ => {}
        }

        // calculate skew_time according to RFC3768 6.1
        let skew_time: f32 = (256.0 - prio as f32) / 256.0;

        // return the newly built VirtualRouter
        Ok(VirtualRouter {
            states: fsm::States::Init,
            parameters: fsm::Parameters::new(
                vrid,
                ifname,
                ifindex,
                prio,
                vip,
                v4addrs,
                v4masks,
                advertint,
                skew_time,
                (3.0 * advertint as f32) + skew_time,
                preempt,
                rfc3768,
                auth_type,
                [0; 8],
                auth_secret,
                protocols,
                netdrv,
            ),
            // initialize the timers
            timers: fsm::Timers::new(5.0, 1),
            // initialize the flags to 0x1 (down flag set)
            flags: fsm::Flags::new(0x1),
            // initialize the protocols
        })
    }
    // is_owner_vip() method
    // check is the VirtualRouter is the owner of the VIP
    pub fn is_owner_vip(&self, vip: &[u8; 4]) -> bool {
        if self.parameters.ipaddrs().contains(vip) {
            true
        } else {
            false
        }
    }
}

// open_raw_socket_fd() function
/// Open a raw AF_PACKET socket for IPv4
fn open_raw_socket_fd() -> io::Result<i32> {
    unsafe {
        // man 2 socket
        // returns a file descriptor or -1 if error.
        match socket(AF_PACKET, SOCK_RAW, ETHER_P_IP.to_be() as i32) {
            -1 => Err(io::Error::last_os_error()),
            fd => Ok(fd),
        }
    }
}

// listen_ip_pkts() function
/// Listen for IP packets
///
/// Library entry point for Virtual Router functions
pub fn listen_ip_pkts(cfg: &Config, shutdown: Arc<AtomicBool>) -> io::Result<()> {
    // initialize sockaddr and packet buffer
    let mut sockaddr: sockaddr_ll = unsafe { mem::zeroed() };
    let mut pkt_buf: [u8; 1024] = [0; 1024];

    // read operation mode
    match cfg.mode {
        // sniffer mode
        0 => {
            // open raw socket
            let sockfd = open_raw_socket_fd()?;

            // get interface name
            let iface = cfg.iface();

            // set promiscuous flag on interface
            let iface = CString::new(iface.as_bytes() as &[u8]).unwrap();
            if let Err(e) = os::linux::netdev::set_if_promiscuous(sockfd, &iface, PflagOp::Set) {
                return Err(e);
            }

            // print information
            println!("Listening for VRRPv2 packets on {}\n", cfg.iface());

            // starts loop
            loop {
                // check if global shutdown variable is set
                // if set, then call set_if_promiscuous() to remove promisc mode on interface
                if shutdown.load(Ordering::Relaxed) {
                    if let Err(e) =
                        os::linux::netdev::set_if_promiscuous(sockfd, &iface, PflagOp::Unset)
                    {
                        return Err(e);
                    } else {
                        println!("Exiting...");
                        std::process::exit(0);
                    }
                }

                // Block on receiving IP packets
                match recv_ip_pkts(sockfd, &mut sockaddr, &mut pkt_buf) {
                    Ok(len) => filter_vrrp_pkt(sockfd, sockaddr, &pkt_buf[0..len]),
                    Err(e) => return Err(e),
                }
            }
        }
        // virtual router modes
        1 | 2 => {
            // read configuration file
            let config = decode_config(cfg.conf());

            // read debugging level from Config first
            let debug_level = match cfg.debug() {
                Some(v) => v,
                // if None, then read debug level from configuration file
                None => config.debug(),
            };

            // initialize 'debug' variable of type Verbose
            // and pass time format string from configuration file
            let debug: Verbose =
                Verbose::new(debug_level, config.time_zone(), config.time_format());

            // if the mode is 2, then daemonize:
            if cfg.mode == 2 {
                // create log files
                let stdout = File::create(config.main_log()).unwrap();
                let stderr = File::create(config.error_log()).unwrap();
                // initialize the daemon
                let deamon = Daemonize::new()
                    .pid_file(config.pid())
                    .chown_pid_file(true)
                    .working_directory(config.working_dir())
                    .user("root")
                    .group("root")
                    .umask(0o027)
                    .stdout(stdout)
                    .stderr(stderr);
                // daemonize the process
                match deamon.start() {
                    Ok(_) => println!("rVRRPd (v{}) daemon started", RVRRPD_VERSION_STRING),
                    Err(e) => eprintln!("Error while starting rVRRPd daemon: {}", e),
                }
            }

            // initialize the virtual router vector
            let mut vrouters: Vec<Arc<RwLock<VirtualRouter>>> = Vec::new();

            // initialize internal protocols structure
            let mut protocols = Protocols { r#static: None };

            // read protocols configuration (if any)
            match config.protocols {
                // if protocols config definition exists
                Some(proto) => {
                    match proto.r#static {
                        // if static routes exists
                        Some(st) => {
                            let mut static_vec: Vec<Static> = Vec::with_capacity(st.len());
                            for s in st {
                                // push static route to the fixed-size vector
                                static_vec.push(Static::new(
                                    s.route(),
                                    s.mask(),
                                    s.nh(),
                                    s.metric(),
                                    s.mtu(),
                                ));
                            }
                            // set static routes
                            protocols.r#static = Some(static_vec);
                        }
                        None => {}
                    }
                }
                None => {}
            }
            // print debugging information
            print_debug(
                &debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_PROTO,
                format!("reading protocols structure - {:?}", protocols),
            );
            // create a RwLock mutex for protocols
            let protocols = Mutex::new(protocols);
            // create an atomic reference count
            let protocols = Arc::new(protocols);

            // check if at least one VR exists
            let vcvr = match config.vrouter {
                Some(vr) => vr,
                None => {
                    eprintln!("error(main): no virtual router configured. exiting...");
                    std::process::exit(1);
                }
            };

            // create a new virtual router and push it into the 'vrouters' vector
            for vr in vcvr {
                // clone protocols
                let protocols = Arc::clone(&protocols);

                // create new virtual router structure
                match VirtualRouter::new(
                    vr.group(),
                    vr.interface().to_string(),
                    vr.priority(),
                    vr.vip(),
                    vr.timer_advert(),
                    vr.preemption(),
                    vr.rfc3768(),
                    vr.auth_type(),
                    vr.auth_secret().clone(),
                    protocols,
                    &debug,
                    vr.netdrv(),
                ) {
                    Ok(vr) => {
                        let vr = RwLock::new(vr);
                        vrouters.push(Arc::new(vr));
                    }
                    Err(e) => return Err(e),
                }
            }

            // open raw socket
            let sockfd = open_raw_socket_fd()?;

            // set vr's interface(s) in promiscuous mode
            for vr in &vrouters {
                // acquire read lock
                let vr = vr.read().unwrap();
                let iface = CString::new(vr.parameters.interface().as_bytes() as &[u8]).unwrap();
                if let Err(e) = os::linux::netdev::set_if_promiscuous(sockfd, &iface, PflagOp::Set)
                {
                    return Err(e);
                }
            }

            // print debugging information
            print_debug(
                &debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_VR,
                format!("created virtual-router vector - {:?}", vrouters),
            );

            // create a pool of threads
            let mut threads = ThreadPool::new(&vrouters, sockfd, &debug);

            // send Startup event to worker threads
            std::thread::sleep(std::time::Duration::from_secs(1));
            threads.startup(&vrouters, &debug);

            loop {
                // check if global shutdown variable is set
                // if set, then call set_if_promiscuous() to remove promisc mode on interface
                if shutdown.load(Ordering::Relaxed) {
                    for vr in &vrouters {
                        // acquire read lock
                        let vr = vr.read().unwrap();
                        let iface =
                            CString::new(vr.parameters.interface().as_bytes() as &[u8]).unwrap();
                        if let Err(e) =
                            os::linux::netdev::set_if_promiscuous(sockfd, &iface, PflagOp::Unset)
                        {
                            return Err(e);
                        }
                    }
                    println!("Exiting...");
                    // Manually calling the threads pool desctructor
                    threads.drop(&vrouters, &debug);
                    std::process::exit(0);
                }

                // Block on receiving IP packets
                match recv_ip_pkts(sockfd, &mut sockaddr, &mut pkt_buf) {
                    Ok(len) => {
                        match verify_vrrp_pkt(sockfd, sockaddr, &pkt_buf[0..len], &vrouters, &debug)
                        {
                            Some((ifindex, vrid, ipsrc, advert_prio)) => {
                                handle_vrrp_advert(
                                    &vrouters,
                                    ifindex,
                                    vrid,
                                    ipsrc,
                                    advert_prio,
                                    &debug,
                                );
                            }
                            _ => (),
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
        }
        _ => {
            println!("Unknown operation mode specified.");
            std::process::exit(1);
        }
    }
}

// recv_ip_pkts() function
/// Receive IP packets
fn recv_ip_pkts(sockfd: i32, sockaddr: &mut sockaddr_ll, buf: &mut [u8]) -> io::Result<usize> {
    // stack variables
    let len: isize;
    let mut addr_buf_len: socklen_t = mem::size_of::<sockaddr_ll>() as socklen_t;

    unsafe {
        // unsafe transmut of sockaddr_ll to a sockaddr type
        let addr_ptr: *mut sockaddr = mem::transmute::<*mut sockaddr_ll, *mut sockaddr>(sockaddr);
        // unsafe call to libc's recvfrom (man 2 recvfrom)
        // returns length of message, -1 if error
        len = match recvfrom(
            sockfd,                          // socket file descriptor
            buf.as_mut_ptr() as *mut c_void, // pointer to buffer
            buf.len(),                       // buffer length
            0,                               // flags
            addr_ptr as *mut sockaddr,       // pointer to source address
            &mut addr_buf_len,               // address buffer length
        ) {
            -1 => {
                return Err(io::Error::last_os_error());
            }
            len => len,
        }
    }

    Ok(len as usize)
}

// verify_vrrp_pkt() function
/// Verify VRRPv2 ADVERTISEMENT packets (as per RFC3768 7.1)
fn verify_vrrp_pkt(
    _sockfd: i32,
    sockaddr: sockaddr_ll,
    packet: &[u8],
    vrouters: &Vec<Arc<RwLock<VirtualRouter>>>,
    debug: &Verbose,
) -> Option<(i32, u8, [u8; 4], u8)> {
    // ignore packets that are too short (plus one IP address and auth. data. field)
    if packet.len() < (mem::size_of::<VRRPpkt>() + 4 + 8) {
        return None;
    }

    // read the *possibly* VRRP packet
    let vrrp_pkt: VRRPpkt = unsafe { ptr::read(packet.as_ptr() as *const _) };

    // filter out all IP packets with IP protocol not matching VRRPv2
    if *vrrp_pkt.ipproto() != IP_UPPER_PROTO_VRRP {
        return None;
    }

    // verify the IP ttl is 255
    if *vrrp_pkt.ipttl() != IP_TTL_VRRP_MINTTL {
        return None;
    }

    // verify the VRRP version is 0x2 and the message type is 0x1 (ADVERTISEMENT)
    if *vrrp_pkt.version() != VRRP_V2_VER_TYPE_AUTHMSG {
        return None;
    }

    // compute the number of bytes to read for the IP addresses
    let ip_bcnt = (vrrp_pkt.s_addrcount(packet.len()) * 4) as usize;

    // construct a variable-size vector of u8 for the VRRP PDU
    let mut vrrp_pdu = Vec::new();
    vrrp_pdu.push(*vrrp_pkt.version());
    vrrp_pdu.push(*vrrp_pkt.vrid());
    vrrp_pdu.push(*vrrp_pkt.prio());
    vrrp_pdu.push(*vrrp_pkt.addrcount());
    vrrp_pdu.push(*vrrp_pkt.authtype());
    vrrp_pdu.push(*vrrp_pkt.adverint());
    vrrp_pdu.push(*vrrp_pkt.checksum() as u8);
    vrrp_pdu.push((*vrrp_pkt.checksum()).to_be() as u8);

    // read IP addresses from packet buffer
    // and extend vrrp_pdu vector to ip addresses
    let ipaddrs = unsafe {
        slice::from_raw_parts(
            packet[ETHER_VRRP_IPADDR_POS..].as_ptr() as *const _,
            ip_bcnt,
        )
    };
    vrrp_pdu.extend_from_slice(&ipaddrs);

    // read authentication data from packet buffer
    // and extend vrrp_pdu vector to auth. data
    let authdata = unsafe {
        slice::from_raw_parts(
            packet[ETHER_VRRP_IPADDR_POS + ip_bcnt..].as_ptr() as *const _,
            8,
        )
    };
    vrrp_pdu.extend_from_slice(&authdata);

    // verify the VRRP checksum (RFC1071)
    if checksums::rfc1071(&vrrp_pdu) != 0xFFFF {
        return None;
    }

    // verify there is an existing vrouter (matching vrid) on the receiving interface
    // and the local router is not the owner of the destination IP address.
    let ifb_vr = vrouters.iter().find(|&v| {
        let v = v.read().unwrap();
        (v.parameters.ifindex() == sockaddr.sll_ifindex)
            && (v.parameters.vrid() == *vrrp_pkt.vrid())
    });
    match ifb_vr {
        // if a virtual router exists for this interface / VRID pair:
        Some(vr) => {
            // first get read lock on vr's RwLock guard
            let vr = vr.read().unwrap();

            // verify the destination address is not owned by the virtual router
            if vr.parameters.ipaddrs().contains(vrrp_pkt.ipdst()) {
                print_debug(
                    debug,
                    DEBUG_LEVEL_MEDIUM,
                    DEBUG_SRC_MAIN,
                    format!("received a VRRP message for an owned IP address"),
                );
                return None;
            }

            // verify the authentication type matches the configured method
            // for this virtual router
            if *vrrp_pkt.authtype() != vr.parameters.authtype() {
                print_debug(
                    debug,
                    DEBUG_LEVEL_MEDIUM,
                    DEBUG_SRC_MAIN,
                    format!("received a VRRP message with a non-matching authentication type"),
                );
                return None;
            }

            // perform message authentication
            match vr.parameters.authtype() {
                // AUTH_TYPE_SIMPLE (RFC2338 Type-1 Plain)
                AUTH_TYPE_SIMPLE => {
                    print_debug(
                        debug,
                        DEBUG_LEVEL_EXTENSIVE,
                        DEBUG_SRC_AUTH,
                        format!("performing VRRP simple (type-1) authentication"),
                    );
                    let d = gen_auth_data(
                        AUTH_TYPE_SIMPLE,
                        vr.parameters.authsecret(),
                        Option::Some(&vrrp_pdu[..vrrp_pdu.len() - 8]),
                    );
                    if d != authdata {
                        print_debug(
                            debug,
                            DEBUG_LEVEL_MEDIUM,
                            DEBUG_SRC_AUTH,
                            format!("VRRP message authentication failed"),
                        );
                        return None;
                    }
                }
                // AUTH_TYPE_P0 (PROPRIETARY-TRUNCATED-8B-SHA256)
                // AUTH_TYPE_P1 (PROPRIETARY-XOF-8B-SHAKE256)
                AUTH_TYPE_P0 | AUTH_TYPE_P1 => {
                    print_debug(
                        debug,
                        DEBUG_LEVEL_EXTENSIVE,
                        DEBUG_SRC_AUTH,
                        format!(
                            "performing VRRP proprietary ({}) authentication",
                            vr.parameters.authtype()
                        ),
                    );
                    // get the verification code on the VRRP PDU minus the authentication header
                    // and the checksum field zero-ed out (HMAC-then-checksum)
                    let zchecksum = [0u8, 0u8];
                    vrrp_pdu.splice(
                        VRRP_V2_CHECKSUM_POS..VRRP_V2_CHECKSUM_POS + 2,
                        zchecksum.iter().cloned(),
                    );
                    let hmac = gen_auth_data(
                        vr.parameters.authtype(),
                        vr.parameters.authsecret(),
                        Option::Some(&vrrp_pdu[..vrrp_pdu.len() - 8]),
                    );
                    // print debugging information
                    print_debug(
                        debug,
                        DEBUG_LEVEL_EXTENSIVE,
                        DEBUG_SRC_AUTH,
                        format!("VRRP message authentication data {:02x?}", &hmac[..]),
                    );
                    // check if authentication data matches
                    if hmac != authdata {
                        print_debug(
                            debug,
                            DEBUG_LEVEL_MEDIUM,
                            DEBUG_SRC_AUTH,
                            format!("VRRP message authentication failed"),
                        );
                        return None;
                    }
                }
                // skip authentication
                _ => {}
            }

            // verify the message's 'avertint' field matches the locally
            // configured vr's advertisement interval
            if *vrrp_pkt.adverint() != vr.parameters.adverint() {
                print_debug(
                    debug,
                    DEBUG_LEVEL_MEDIUM,
                    DEBUG_SRC_MAIN,
                    format!("received a VRRP message with a non-matching advertisement interval"),
                );
                return None;
            }

            // return the vr's ifindex, the vrid and advertisement's priority to the caller function
            Some((
                vr.parameters.ifindex(),
                vr.parameters.vrid(),
                *vrrp_pkt.ipsrc(),
                *vrrp_pkt.prio(),
            ))
        }
        // if no matching virtual router exists, simply drop the VRRP message
        None => {
            print_debug(
                debug,
                DEBUG_LEVEL_MEDIUM,
                DEBUG_SRC_MAIN,
                format!("received a VRRP message for a non-existing virtual router"),
            );
            return None;
        }
    }
}

// handle_vrrp_advert() function
/// Handle VRRPv2 ADVERTISEMENT message
fn handle_vrrp_advert(
    vrouters: &Vec<Arc<RwLock<VirtualRouter>>>,
    ifindex: i32,
    vrid: u8,
    ipsrc: [u8; 4],
    advert_prio: u8,
    debug: &Verbose,
) {
    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_MEDIUM,
        DEBUG_SRC_MAIN,
        format!(
            "got a valid VRRPv2 packet for VRID {} on if {}",
            vrid, ifindex
        ),
    );

    // find matching virtual router instance
    let ifb_vr = vrouters.iter().find(|&v| {
        let v = v.read().unwrap();
        (v.parameters.ifindex() == ifindex) && (v.parameters.vrid() == vrid)
    });

    match ifb_vr {
        // a virtual router does match the ADVERTISEMENT
        Some(vr) => {
            // get read lock (again)
            let vr = vr.read().unwrap();
            // if the channel is registered, acquire lock and notify the VR of the message receipt
            match vr.parameters.notification() {
                Some(tx) => {
                    // print debugging information
                    print_debug(
                        debug,
                        DEBUG_LEVEL_EXTENSIVE,
                        DEBUG_SRC_MAIN,
                        format!("sending Advert event notification"),
                    );
                    // acquiring lock on sender channel
                    tx.lock()
                        .unwrap()
                        .send(fsm::Event::Advert(ipsrc, advert_prio))
                        .unwrap();
                    // print debugging information
                    print_debug(
                        debug,
                        DEBUG_LEVEL_EXTENSIVE,
                        DEBUG_SRC_MAIN,
                        format!("Advert event notification sent"),
                    );
                }
                None => print_debug(
                    debug,
                    DEBUG_LEVEL_LOW,
                    DEBUG_SRC_MAIN,
                    format!("got ADVERTISEMENT message while notification channel not ready"),
                ),
            }
        }
        None => {
            panic!("error(main): incorrect virtual router reference, possible race condition, panicking");
        }
    }
}

// filter_vrrp_pkt() function
/// Filter VRRPv2 packets for sniffing mode
fn filter_vrrp_pkt(sockfd: i32, _sockaddr: sockaddr_ll, packet: &[u8]) {
    // ignore packets that are way too short (plus auth. data. field)
    if packet.len() < (mem::size_of::<VRRPpkt>() + 8) {
        return;
    }

    // read packet
    let vrrp_pkt: VRRPpkt = unsafe { ptr::read(packet.as_ptr() as *const _) };

    // filter VRRP packets (IP Proto 112)
    if *vrrp_pkt.ipproto() != IP_UPPER_PROTO_VRRP {
        return;
    }

    // verify the IP TTL is 255 (per RFC3768 7.1)
    if *vrrp_pkt.ipttl() != IP_TTL_VRRP_MINTTL {
        println!(
            "VRRP message received with invalid TTL {:#X}.",
            vrrp_pkt.ipttl()
        );
    }

    // perform VRRP sanity checks
    // if VRRP version is not 2 and type is not advertisement (p/x 0b00100001)
    if *vrrp_pkt.version() != VRRP_V2_VER_TYPE_AUTHMSG {
        return;
    }

    // compute the number of bytes to read for the IP addresses
    let ip_bcnt = (vrrp_pkt.s_addrcount(packet.len()) * 4) as usize;

    // constructing a variable-size vector of u8 for the VRRP PDU
    let mut vrrp_pdu = Vec::new();
    vrrp_pdu.push(*vrrp_pkt.version());
    vrrp_pdu.push(*vrrp_pkt.vrid());
    vrrp_pdu.push(*vrrp_pkt.prio());
    vrrp_pdu.push(*vrrp_pkt.addrcount());
    vrrp_pdu.push(*vrrp_pkt.authtype());
    vrrp_pdu.push(*vrrp_pkt.adverint());
    vrrp_pdu.push(*vrrp_pkt.checksum() as u8);
    vrrp_pdu.push((*vrrp_pkt.checksum()).to_be() as u8);

    // read IP addresses from packet buffer
    // and extend vrrp_pdu vector to ip addresses
    let ipaddrs = unsafe {
        slice::from_raw_parts(
            packet[ETHER_VRRP_IPADDR_POS..].as_ptr() as *const _,
            ip_bcnt,
        )
    };
    vrrp_pdu.extend_from_slice(&ipaddrs);

    // read authentication data from packet buffer
    // and extend vrrp_pdu vector to auth. data
    let authdata = unsafe {
        slice::from_raw_parts(
            packet[ETHER_VRRP_IPADDR_POS + ip_bcnt..].as_ptr() as *const _,
            8,
        )
    };
    vrrp_pdu.extend_from_slice(&authdata);

    // verify result of the RFC1071 checksum
    if checksums::rfc1071(&vrrp_pdu) != 0xFFFF {
        println!(
            "VRRP message with invalid checksum {:#X} detected",
            checksums::rfc1071(&vrrp_pdu)
        );
    }

    // call show_vrrp_pkt() to handle VRRPv2 packets
    show_vrrp_pkt(sockfd, &vrrp_pkt, ipaddrs, authdata);
}

// show_vrrp_pkt() function
/// Display VRRPv2 packets
fn show_vrrp_pkt(_sockfd: i32, vrrp_pkt: &VRRPpkt, ipaddrs: &[u8], _authdata: &[u8]) {
    // prints some fields
    println!("VRRPv2 Packet:");
    println!(" Version/Type: {:#2X}", vrrp_pkt.version());
    println!(" Virtual Router ID: {}", vrrp_pkt.vrid());
    println!(" Priority: {}", vrrp_pkt.prio());
    println!(" IP Address Count: {}", vrrp_pkt.addrcount());
    println!(" Authentication Type: {:#2X}", vrrp_pkt.authtype());
    println!(" Advertisement Interval: {}s", vrrp_pkt.adverint());
    println!(" VRRP Checksum: {:#X}", vrrp_pkt.checksum());
    println!(" IP Address(es):");
    for (a, b, c, d) in ipaddrs.into_iter().tuple_windows() {
        println!("  - {}.{}.{}.{}\n", a, b, c, d)
    }
}
