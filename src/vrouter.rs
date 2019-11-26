//! Virtual Router module
//!
use crate::*;

// libc
#[cfg(target_os = "freebsd")]
use crate::os::freebsd::libc::raw_sendto;
#[cfg(target_os = "linux")]
use crate::os::linux::libc::raw_sendto;
#[cfg(target_os = "freebsd")]
use libc::{c_void, write};
#[cfg(target_os = "linux")]
use libc::{sendto, AF_PACKET};

// debugging
use crate::debug::Verbose;

// packets related function
use crate::packets::as_u8_slice;

// operating system drivers
use crate::os::drivers::Operation;

// address resolution protocol
#[cfg(target_os = "freebsd")]
use crate::os::freebsd::arp;
#[cfg(target_os = "linux")]
use crate::os::linux::arp;

/// Virtual Router Structure
#[derive(Debug)]
pub struct VirtualRouter {
    pub parameters: Parameters,
    states: fsm::States,
    pub timers: fsm::Timers,
    pub flags: fsm::Flags,
}

// VirtualRouter Type Implementation
impl VirtualRouter {
    // new() method
    // create new VirtualRouter
    pub fn new(
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
        iftype: IfTypes,
        vif_name: String,
        fd: i32,
        socket_filter: bool,
    ) -> io::Result<VirtualRouter> {
        // --- Linux specific interface handling
        #[cfg(target_os = "linux")]
        // get ifindex from interface name
        let ifindex = match os::linux::libc::c_ifnametoindex(&ifname) {
            Ok(i) => i as i32,
            Err(e) => return Err(e),
        };
        // END Linux specific interface handling

        // -- FreeBSD specific interface handling
        #[cfg(target_os = "freebsd")]
        let ifindex = -1;
        // END FreeBSD specific interface handling

        // create new IPv4 addresses vector
        let mut v4addrs = Vec::new();

        // create new IPv4 netmasks vector
        let mut v4masks = Vec::new();

        // build interface IPv4 addresses list
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        {
            let _r = os::multi::libc::get_addrlist(&ifname, &mut v4addrs, &mut v4masks);
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
            parameters: Parameters::new(
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
                iftype,
                vif_name,
                0,
                fd,
                socket_filter,
            ),
            // initialize the timers
            timers: fsm::Timers::new(5.0, 1),
            // initialize the flags to 0x1 (down flag set)
            flags: fsm::Flags::new(0x1),
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
    // states() getter
    pub fn get_states(&self) -> &fsm::States {
        &self.states
    }
    // states() setter
    pub fn set_states(&mut self, s: fsm::States) {
        self.states = s;
    }

    // send_advertisement() method
    /// Send a VRRP ADVERTISEMENT message
    pub fn send_advertisement(&self, fd: i32, debug: &Verbose) -> io::Result<()> {
        // generate initial VRRP ADVERTISEMENT frame/packet
        let advert = VRRPpkt::gen_advert(self);

        // build static frame slice
        let static_frame = unsafe { as_u8_slice(&advert) };

        // initialize frame_vec vector and push static frame into it
        let mut frame: Vec<u8> = Vec::new();
        for b in static_frame {
            frame.push(*b);
        }

        // set and push the VIP to the ipaddrs
        let vip = self.parameters.vip();
        for i in 0..4 {
            frame.push(vip[i]);
        }

        // check if rfc3768 compatibility flag is true
        if !self.parameters.rfc3768() {
            // extend the frame with the variable-length list of local IP addresses
            for addr in self.parameters.ipaddrs() {
                for i in 0..4 {
                    frame.push(addr[i]);
                }
            }
        }

        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_PACKET,
            format!(
                "sending advertisement frame out if {}, {:?}",
                self.parameters.interface(),
                frame
            ),
        );

        // add authentication data
        match self.parameters.authtype() {
            // AUTH_TYPE_P0 (PROPRIETARY-TRUNCATED-8B-SHA256)
            // AUTH_TYPE_P1 (PROPRIETARY-XOF-8B-SHAKE256)
            AUTH_TYPE_P0 | AUTH_TYPE_P1 => {
                for b in gen_auth_data(
                    self.parameters.authtype(),
                    self.parameters.authsecret(),
                    Option::Some(&frame[VRRP_V2_FRAME_OFFSET..]),
                ) {
                    frame.push(b);
                }
            }
            // all remaining types
            _ => {
                for b in gen_auth_data(
                    self.parameters.authtype(),
                    self.parameters.authsecret(),
                    Option::None,
                ) {
                    frame.push(b);
                }
            }
        }

        // generate VRRP checksum (vrrp checksum is at offset 34+6 bytes)
        let vrrp_checksum =
            checksums::one_complement_sum(&frame[VRRP_V2_FRAME_OFFSET..], Option::Some(6));
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_PACKET,
            format!("VRRP checksum is {:#X}", vrrp_checksum),
        );
        // set vrrp's checksum field
        frame[VRRP_V2_FRAME_OFFSET + 6] = vrrp_checksum.to_be() as u8;
        frame[VRRP_V2_FRAME_OFFSET + 6 + 1] = vrrp_checksum as u8;

        // generate IP checksum (ip checksum is at offset 14+10 bytes)
        let ip_checksum =
            checksums::one_complement_sum(&frame[IP_FRAME_OFFSET..], Option::Some(10));
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_PACKET,
            format!("IP checksum is {:#X}", ip_checksum),
        );

        // set ip checksum field (offset 34)
        frame[IP_FRAME_OFFSET + 10] = ip_checksum.to_be() as u8;
        frame[IP_FRAME_OFFSET + 10 + 1] = ip_checksum as u8;

        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_PACKET,
            format!(
                "final ADVERTISEMENT frame is {} bytes long",
                frame.len() - ETHER_FRAME_SIZE
            ),
        );
        // set length of ip packet (offset 16)
        // the length of ip header + data = frame size - ethernet frame
        let frame_size = frame.len() - ETHER_FRAME_SIZE;
        frame[IP_FRAME_OFFSET + 2] = frame_size.to_be() as u8;
        frame[IP_FRAME_OFFSET + 2 + 1] = frame_size as u8;

        // sending raw ethernet frame
        let ifindex = self.parameters.ifindex();
        let res = raw_sendto(fd, ifindex, &mut frame, &debug);

        // return above call result
        return res;
    }

    // broadcast_gratuitious_arp() function
    /// Broadcast Gratuitious ARP requests
    pub fn broadcast_gratuitious_arp(&self, fd: i32, debug: &Verbose) -> io::Result<()> {
        // suppress warnings about usued variables on linux
        #[cfg(target_os = "linux")]
        let _d = debug;

        // build gratuitious ARP request
        let mut arpframe = arp::ARPframe {
            dst_mac: ETHER_ARP_DST_MAC,
            src_mac: ETHER_VRRP_V2_SRC_MAC,
            ethertype: ETHER_P_ARP.to_be(),

            hardware_type: ARP_HW_TYPE.to_be(),
            protocol_type: ETHER_P_IP.to_be(),
            hw_addr_len: 6,
            proto_addr_len: 4,
            opcode: ARP_OP_REQUEST.to_be(),
            sender_hw_addr: ETHER_VRRP_V2_SRC_MAC,
            sender_proto_addr: self.parameters.vip(),
            target_hw_addr: [0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            target_proto_addr: [255, 255, 255, 255],
        };

        // --- Linux specific handling
        #[cfg(target_os = "linux")]
        {
            // set VRID on source MAC addresses
            arpframe.src_mac[5] = self.parameters.vrid();
            arpframe.sender_hw_addr[5] = self.parameters.vrid();

            // sockaddr_ll (man 7 packet)
            let mut sa = sockaddr_ll {
                sll_family: AF_PACKET as u16,
                sll_protocol: ETHER_P_ARP.to_be(),
                sll_ifindex: self.parameters.ifindex(),
                sll_hatype: 0,
                sll_pkttype: 0,
                sll_halen: 0,
                sll_addr: [0; 8],
            };

            unsafe {
                let ptr_sockaddr = mem::transmute::<*mut sockaddr_ll, *mut sockaddr>(&mut sa);
                match sendto(
                    fd,
                    &mut arpframe as *mut _ as *const c_void,
                    mem::size_of_val(&arpframe),
                    0,
                    ptr_sockaddr,
                    mem::size_of_val(&sa) as u32,
                ) {
                    -1 => Err(io::Error::last_os_error()),
                    _ => Ok(()),
                }
            }
        }
        // END Linux specific handling

        // --- FreeBSD specific handling
        #[cfg(target_os = "freebsd")]
        {
            // set VRID on source MAC addresses
            arpframe.src_mac[5] = self.parameters.vrid();
            arpframe.sender_hw_addr[5] = self.parameters.vrid();

            unsafe {
                // unsafe call to write()
                match write(
                    fd,
                    &mut arpframe as *mut _ as *const c_void,
                    mem::size_of_val(&arpframe),
                ) {
                    -1 => Err(io::Error::last_os_error()),
                    _ => {
                        print_debug(
                            debug,
                            DEBUG_LEVEL_MEDIUM,
                            DEBUG_SRC_ARP,
                            format!("VRRPv2 frame successfully sent on BPF device, fd {}", fd),
                        );
                        Ok(())
                    }
                }
            }
        }
        // END FreeBSD specific handling
    }

    // set_ip_addresses() method
    /// set or clear IPv4 addresses on a virtual-router interface
    pub fn set_ip_addresses(&self, fd: i32, op: Operation, debug: &Verbose) {
        // create addr and netmask vector
        let mut addrs: Vec<[u8; 4]> = Vec::new();
        let mut netmasks: Vec<[u8; 4]> = Vec::new();

        // push vr's address(es) inside addr vector
        for ip in self.parameters.ipaddrs() {
            addrs.push(*ip);
        }
        // push address' netmasks inside the netmask vector
        for m in self.parameters.ipmasks() {
            netmasks.push(*m);
        }

        // if true, add vip and netmask to the respective vectors
        // make sure this is done last, so the VIP is on top of the addrs vector
        // otherwise it will never replace the current IP when using ioctls
        match op {
            Operation::Add => {
                // add vip to the IP addresses vector
                addrs.push(self.parameters.vip());
                // add first address' netmask
                netmasks.push(self.parameters.ipmasks()[0]);
            }
            _ => {}
        }

        // construct interface name
        let ifname = CString::new(self.parameters.interface().as_bytes() as &[u8]).unwrap();

        // set the last ip address from vector
        let idx = addrs.len() - 1;

        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_HIGH,
            DEBUG_SRC_IP,
            format!(
                "setting IP address {}.{}.{}.{} netmask {}.{}.{}.{} on {:?}",
                addrs[idx][0],
                addrs[idx][1],
                addrs[idx][2],
                addrs[idx][3],
                netmasks[idx][0],
                netmasks[idx][1],
                netmasks[idx][2],
                netmasks[idx][3],
                ifname
            ),
        );

        // --- Linux specific interface tyoe handling
        #[cfg(target_os = "linux")]
        {
            let ifindex = match self.parameters.iftype() {
                IfTypes::macvlan => self.parameters.vifidx(),
                _ => self.parameters.ifindex(),
            };

            // set ifindex on physical or macvlan interface
            // set virtual ip address according to the network driver in use
            match self.parameters.netdrv() {
                NetDrivers::ioctl => {
                    if let Err(e) =
                        os::linux::netdev::set_ip_address(fd, &ifname, addrs[idx], netmasks[idx])
                    {
                        eprintln!(
                            "error(ip): error while assigning IP address on interface {:?}: {}",
                            &ifname, e
                        );
                    }
                }
                NetDrivers::libnl => {
                    print_debug(
                        debug,
                        DEBUG_LEVEL_HIGH,
                        DEBUG_SRC_IP,
                        format!(
                    "setting up IP address on interface {:?} (ifindex: {}) using netlink (libnl)",
                    &ifname, ifindex
                ),
                    );
                    if let Err(e) = os::linux::libnl::set_ip_address(
                        ifindex,
                        &ifname,
                        addrs[idx],
                        netmasks[idx],
                        Operation::Add,
                        debug,
                    ) {
                        eprintln!(
                            "error(ip): error while assigning IP address on interface {:?}: {}",
                            &ifname, e
                        );
                    }
                }
            }
        }
        // END Linux specific interface type handling

        // FreeBSD specific interface type handling
        #[cfg(target_os = "freebsd")]
        {
            print_debug(
                debug,
                DEBUG_LEVEL_HIGH,
                DEBUG_SRC_IP,
                format!("setting ip addresss on interface {:?}, fd {}", ifname, fd),
            );
            if let Err(e) = os::freebsd::netinet::set_ip_address(
                fd,
                &ifname,
                addrs[idx],
                netmasks[idx],
                Operation::Add,
            ) {
                eprintln!(
                    "error(ip): error while setting IP address on interface {:?}: {}",
                    ifname, e
                );
            }
        }
        // END FreeBSD specific interface type handling
    }

    // delete_ip_addresses() method
    /// delete an ip address on a virtual-router interface
    pub fn delete_ip_addresses(&self, fd: i32, debug: &Verbose) {
        // create netmasks vector
        let mut netmasks: Vec<[u8; 4]> = Vec::new();

        // push first ip's netmask
        // always take the mask of the primary (top) ip
        netmasks.push(self.parameters.ipmasks()[0]);

        // construct interface name
        let ifname = CString::new(self.parameters.interface().as_bytes() as &[u8]).unwrap();

        // print debugging information
        let vip = self.parameters.vip();
        print_debug(
            debug,
            DEBUG_LEVEL_HIGH,
            DEBUG_SRC_IP,
            format!(
                "removing IP address {}.{}.{}.{} netmask {}.{}.{}.{} on {:?}",
                vip[0],
                vip[1],
                vip[2],
                vip[3],
                netmasks[0][0],
                netmasks[0][1],
                netmasks[0][2],
                netmasks[0][3],
                ifname
            ),
        );

        // --- Linux specific interface tyoe handling
        #[cfg(target_os = "linux")]
        {
            // workaround compilation warning
            let _fd = fd;
            // delete virtual ip address according to the network driver in use
            match self.parameters.netdrv() {
                NetDrivers::libnl => {
                    print_debug(
                        debug,
                        DEBUG_LEVEL_HIGH,
                        DEBUG_SRC_IP,
                        format!(
                        "removing IP address on interface {:?} (ifindex: {}) using netlink (libnl)",
                        &ifname,
                        self.parameters.ifindex()
                    ),
                    );
                    if let Err(e) = os::linux::libnl::set_ip_address(
                        self.parameters.ifindex(),
                        &ifname,
                        self.parameters.vip(),
                        netmasks[0],
                        Operation::Rem,
                        debug,
                    ) {
                        eprintln!(
                            "error(ip): error while removing IP address on interface {:?}: {}",
                            &ifname, e
                        );
                    }
                }
                _ => {}
            }
        }
        // END Linux specific interface type handling

        // FreeBSD specific interface type handling
        #[cfg(target_os = "freebsd")]
        {
            print_debug(
                debug,
                DEBUG_LEVEL_HIGH,
                DEBUG_SRC_IP,
                format!("setting ip addresss on interface {:?}, fd {}", ifname, fd),
            );
            if let Err(e) = os::freebsd::netinet::set_ip_address(
                fd,
                &ifname,
                self.parameters.vip,
                netmasks[0],
                Operation::Rem,
            ) {
                eprintln!(
                    "error(ip): error while setting IP address on interface {:?}: {}",
                    ifname, e
                );
            }
        }
        // END FreeBSD specific interface type handling
    }

    // get_mac_addresses() method
    /// get Ethernet MAC address from vr's interface
    #[cfg(target_os = "linux")]
    pub fn get_mac_addresses(&self, fd: i32, debug: &Verbose) -> [u8; 6] {
        // --- Linux specific interface tyoe handling
        #[cfg(target_os = "linux")]
        {
            // construct interface name
            let ifname = CString::new(self.parameters.interface().as_bytes() as &[u8]).unwrap();
            // get mac address of interface
            match os::linux::netdev::get_mac_addr(fd, &ifname, debug) {
                Ok(mac) => mac,
                Err(e) => {
                    eprintln!(
                        "error(mac): error while getting MAC address on interface {:?}: {}",
                        ifname, e
                    );
                    [0, 0, 0, 0, 0, 0]
                }
            }
        }
        // END Linux specific interface type handling
    }

    // set_mac_addresses() method
    /// Set Ethernet MAC address on vr's interface
    #[cfg(target_os = "linux")]
    pub fn set_mac_addresses(&self, fd: i32, mac: [u8; 6], debug: &Verbose) {
        // construct interface name
        let ifname = CString::new(self.parameters.interface().as_bytes() as &[u8]).unwrap();

        // --- Linux specific interface tyoe handling
        #[cfg(target_os = "linux")]
        {
            // set mac address
            match os::linux::netdev::set_mac_addr(fd, &ifname, mac, debug) {
                Err(e) => eprintln!("error(mac): error while setting mac address: {}", e),
                _ => {}
            }
        }
        // END Linux specific interface type handling
    }

    // set_ip_routes() method
    /// set or unset IPv4 routes on virtual-router interfaces
    #[cfg(target_os = "linux")]
    pub fn set_ip_routes(&mut self, fd: i32, op: Operation, debug: &Verbose) {
        // acquire mutex lock on protocols
        let protocols = &self.parameters.protocols();
        let protocols = protocols.lock().unwrap();

        // construct interface name
        let ifname = CString::new(self.parameters.interface().as_bytes() as &[u8]).unwrap();

        // ensure routes are added or deleted only once
        match op {
            Operation::Add => {
                if self.flags.rtset() {
                    return; // route already added
                }
            }
            Operation::Rem => {
                if !self.flags.rtset() {
                    return; // remove not needed
                }
            }
        }

        // check if static protocol reference exists
        match protocols.r#static.as_ref() {
            Some(r) => {
                // for every static routes
                for st in r {
                    // --- Linux specific interface tyoe handling
                    #[cfg(target_os = "linux")]
                    {
                        // add route acccording to the network driver in use
                        match self.parameters.netdrv() {
                            NetDrivers::ioctl => {
                                print_debug(
                                debug,
                                DEBUG_LEVEL_HIGH,
                                DEBUG_SRC_IP,
                                format!(
                                "setting up route on interface {:?} (ifindex: {}) using netlink (ioctl)",
                                &ifname, self.parameters.ifindex()
                            ),
                            );
                                if let Err(e) = os::linux::netdev::set_ip_route(
                                    fd,
                                    &self.parameters.interface(),
                                    st.route(),
                                    st.mask(),
                                    st.nh(),
                                    st.metric(),
                                    st.mtu(),
                                    &op,
                                    debug,
                                ) {
                                    eprintln!(
                                        "error(route): cannot add or delete route {:?}: {}",
                                        st.route(),
                                        e
                                    );
                                }
                            }
                            NetDrivers::libnl => {
                                print_debug(
                                debug,
                                DEBUG_LEVEL_HIGH,
                                DEBUG_SRC_IP,
                                format!(
                                "setting up route on interface {:?} (ifindex: {}) using netlink (libnl)",
                                &ifname, self.parameters.ifindex()
                            ),
                            );
                                if let Err(e) = os::linux::libnl::set_ip_route(
                                    fd,
                                    &self.parameters.interface(),
                                    st.route(),
                                    st.mask(),
                                    st.nh(),
                                    st.metric(),
                                    st.mtu(),
                                    &op,
                                    debug,
                                ) {
                                    eprintln!(
                                        "error(route): cannot add or delete route {:?}: {}",
                                        st.route(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                    // END Linux specific interface type handling
                }
            }
            None => {}
        }

        // set rtset flag according to the completed operation
        match op {
            Operation::Add => self.flags.set_rtset(),
            Operation::Rem => self.flags.clear_rtset(),
        }
    }

    // setup_mac_vlan_link() method (Linux specific)
    #[cfg(target_os = "linux")]
    pub fn setup_macvlan_link(
        &self,
        vmac: [u8; 6],
        op: Operation,
        debug: &Verbose,
    ) -> Option<(i32, String)> {
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_HIGH,
            DEBUG_SRC_MACVLAN,
            format!(
                "setting up macvlan interface on master {:?} using libnl",
                self.parameters.interface()
            ),
        );

        // call to libnl setup_macvlan_link()
        match os::linux::libnl::setup_macvlan_link(self, vmac, &op) {
            // the macvlan interface has been added or deleted successfully
            Ok(()) => {
                // If added, return the ifindex and name of the virtual interface
                match op {
                    Operation::Add => {
                        // find new macvlan ifindex
                        match os::linux::libc::c_ifnametoindex(&self.parameters.vifname()) {
                            Ok(i) => {
                                return Some((i as i32, self.parameters.vifname().clone()));
                            }
                            Err(_e) => return None,
                        }
                    }
                    Operation::Rem => {
                        return None;
                    }
                }
            }
            // catched an error while setting up the macvlan interface
            Err(e) => {
                eprintln!(
                "error(macvlan): cannot perform operation {:?} on macvlan interface (master if: {:?}): {}",
                op, self.parameters.interface(), e
            );
                None
            }
        }
    }
}

/// Virtual Router Parameters Structure
#[derive(Debug)]
pub struct Parameters {
    vrid: u8,                    // Virtual Router Identifier (1-255)
    interface: String,           // Interface where the virtual router is running
    ifindex: i32,                // Interface ifindex
    prio: u8,                    // Priority (0-255)
    vip: [u8; 4],                // Virtual IP (not in RFC parameters list)
    ipaddrs: Vec<[u8; 4]>, // One or more local IPv4 Addresse(s) associated with the virtual router
    ipmasks: Vec<[u8; 4]>, // IPv4 Netmask(s) of above IP addresses
    adverint: u8,          // Advertisement interval
    skew_time: f32,        // Time to skew Master_Down interval (second)
    master_down: f32,      // Time interval for Backup to declare Master Down
    preempt_mode: bool, // Control whether a higher-priority Backup router can preempt a lower-priority Master
    rfc3768: bool,      // RFC2338 compatibility flag
    auth_type: u8,      // Authentication type being used
    auth_data: [u8; 8], // Autentication data (type specific)
    auth_secret: Option<String>, // Authentication secret
    notification: Option<Arc<Mutex<mpsc::Sender<fsm::Event>>>>, // Notification channel
    protocols: Arc<Mutex<Protocols>>, // Internal protocols information
    ifmac: [u8; 6],     // Interface Ethernet MAC address
    netdrv: NetDrivers, // Network driver
    iftype: IfTypes,    // Interfaces type
    vif_name: String,   // Virtual interface name (or physical when saved)
    vif_idx: i32,       // Virtual interface ifindex
    fd: i32,            // Raw socket or BPF file descriptor
    socket_filter: bool, // linux socket filter support
}

/// Parameters Type Implementation
impl Parameters {
    // new() method
    pub fn new(
        vrid: u8,
        interface: String,
        ifindex: i32,
        prio: u8,
        vip: [u8; 4],
        ipaddrs: Vec<[u8; 4]>,
        ipmasks: Vec<[u8; 4]>,
        adverint: u8,
        skew_time: f32,
        master_down: f32,
        preempt_mode: bool,
        rfc3768: bool,
        auth_type: u8,
        auth_data: [u8; 8],
        auth_secret: Option<String>,
        protocols: Arc<Mutex<Protocols>>,
        netdrv: NetDrivers,
        iftype: IfTypes,
        vif_name: String,
        vif_idx: i32,
        fd: i32,
        socket_filter: bool,
    ) -> Parameters {
        Parameters {
            vrid,
            interface,
            ifindex,
            prio,
            vip,
            ipaddrs,
            ipmasks,
            adverint,
            skew_time,
            master_down,
            preempt_mode,
            rfc3768,
            auth_type,
            auth_data,
            auth_secret,
            notification: Option::None,
            protocols,
            ifmac: [0, 0, 0, 0, 0, 0],
            netdrv,
            iftype,
            vif_name,
            vif_idx,
            fd,
            socket_filter,
        }
    }
    // vrid() getter
    pub fn vrid(&self) -> u8 {
        self.vrid
    }
    // interface() getter
    pub fn interface(&self) -> String {
        self.interface.clone()
    }
    // set_interface() setter
    #[cfg(target_os = "linux")]
    pub fn set_interface(&mut self, intf: String) {
        self.interface = intf;
    }
    // ifindex() getter
    pub fn ifindex(&self) -> i32 {
        self.ifindex
    }
    // prio() getter
    pub fn prio(&self) -> u8 {
        self.prio
    }
    // set_prio() setter
    pub fn set_prio(&mut self, prio: u8) {
        self.prio = prio;
    }
    // vip() getter
    pub fn vip(&self) -> [u8; 4] {
        self.vip
    }
    // ipaddrs() getter
    pub fn ipaddrs(&self) -> &Vec<[u8; 4]> {
        &self.ipaddrs
    }
    // ipmasks() getter
    pub fn ipmasks(&self) -> &Vec<[u8; 4]> {
        &self.ipmasks
    }
    // adverint() getter
    pub fn adverint(&self) -> u8 {
        self.adverint
    }
    // skewtime() getter
    pub fn skewtime(&self) -> f32 {
        self.skew_time
    }
    // master_down() getter
    pub fn master_down(&self) -> f32 {
        self.master_down
    }
    // preempt() getter
    pub fn preempt(&self) -> bool {
        self.preempt_mode
    }
    // rfc3768() getter
    pub fn rfc3768(&self) -> bool {
        self.rfc3768
    }
    // authtype() getter
    pub fn authtype(&self) -> u8 {
        self.auth_type
    }
    // authsecret() getter
    pub fn authsecret(&self) -> &Option<String> {
        &self.auth_secret
    }
    // addrcount() method
    pub fn addrcount(&self) -> u8 {
        // calculate the number of addresses (or arrays) in ipaddrs vector
        let num = *&self.ipaddrs.len() as u8;
        // if rfc3768 compatibility flag is false, add one to account for the VIP
        if !self.rfc3768 {
            num + 1
        } else {
            num
        }
    }
    // primary_ip() method
    pub fn primary_ip(&self) -> [u8; 4] {
        // return the first array in vector
        self.ipaddrs[0]
    }
    // notification() method    // require review
    pub fn notification(&self) -> &Option<Arc<Mutex<mpsc::Sender<fsm::Event>>>> {
        &self.notification
    }
    // set_notification() setter
    pub fn set_notification(&mut self, chan: Arc<Mutex<mpsc::Sender<fsm::Event>>>) {
        self.notification = Option::Some(chan);
    }
    // protocols() getter // review review
    #[cfg(target_os = "linux")]
    pub fn protocols(&self) -> Arc<Mutex<Protocols>> {
        self.protocols.clone()
    }
    // ifmac() getter
    #[cfg(target_os = "linux")]
    pub fn ifmac(&self) -> [u8; 6] {
        self.ifmac
    }
    // set_ifmac() setter
    #[cfg(target_os = "linux")]
    pub fn set_ifmac(&mut self, mac: [u8; 6]) {
        self.ifmac = mac;
    }
    // netdrv() getter
    #[cfg(target_os = "linux")]
    pub fn netdrv(&self) -> &NetDrivers {
        &self.netdrv
    }
    // iftype() getter
    #[cfg(target_os = "linux")]
    pub fn iftype(&self) -> &IfTypes {
        &self.iftype
    }
    // vif_name() getter
    #[cfg(target_os = "linux")]
    pub fn vifname(&self) -> String {
        self.vif_name.clone()
    }
    // set_vif_name() setter
    #[cfg(target_os = "linux")]
    pub fn set_vifname(&mut self, vif: String) {
        self.vif_name = vif;
    }
    // vifidx() getter
    #[cfg(target_os = "linux")]
    pub fn vifidx(&self) -> i32 {
        self.vif_idx
    }
    // set_vifidx() setter
    #[cfg(target_os = "linux")]
    pub fn set_vifidx(&mut self, idx: i32) {
        self.vif_idx = idx;
    }
    // fd() getter
    pub fn fd(&self) -> i32 {
        self.fd
    }
    // set_fd() method
    pub fn set_fd(&mut self, fd: i32) {
        self.fd = fd;
    }
    // socket_filter() getter
    pub fn socket_filter(&self) -> bool {
        self.socket_filter
    }
}
