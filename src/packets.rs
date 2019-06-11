//! packets handling module
//! This module includes the various packets formats and related functions such as internet checksums calculation function.

// constants
use crate::constants::*;

// channels and threads
use std::sync::RwLockWriteGuard;

// virtual router
use crate::VirtualRouter;

// checksums
use crate::checksums;

// debugging
use crate::debug::{Verbose, print_debug};

// libc
use libc::{c_void, sendto, sockaddr, sockaddr_ll, AF_PACKET};

// std
use std::io;
use std::mem;

/// Raw VRRPv2 Packet Format Structure
/// This is the fixed size portion of a possibly VRRPv2 packet
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VRRPpkt {
    // Ethernet frame headers
    dst_mac: [u8; 6], // destination MAC address
    src_mac: [u8; 6], // source MAC address
    ethertype: u16,   // ether type

    // IPv4 packet headers
    ipver: u8,       // IP version and header length
    ipdscp: u8,      // DSCP
    iplength: u16,   // length
    ipident: u16,    // identifier
    ipflags: u16,    // flags
    ipttl: u8,       // TTL
    ipproto: u8,     // IP Protocol
    ipchecksum: u16, // Header Checksum
    ipsrc: [u8; 4],  // source IP address
    ipdst: [u8; 4],  // destinatin IP address

    // VRRPv2 packet format (RFC3768)
    version: u8,   // version/type - 4/4 bits
    vrid: u8,      // virtual router id - 8 bits
    prio: u8,      // priority - 8 bits
    addrcount: u8, // count ip addr - 8 bits
    authtype: u8,  // auth type - 8 bits
    adverint: u8,  // advertisement interval - 8 bits
    checksum: u16, // checksum - 16 bits
}

// VRRPpkt methods
impl VRRPpkt {
    // getters
    pub fn _dst_mac(&self) -> &[u8; 6] {
        &self.dst_mac
    }
    pub fn _src_mac(&self) -> &[u8; 6] {
        &self.src_mac
    }
    pub fn _ethertype(&self) -> &u16 {
        &self.ethertype
    }
    pub fn ipsrc(&self) -> &[u8; 4] {
        &self.ipsrc
    }
    pub fn ipdst(&self) -> &[u8; 4] {
        &self.ipdst
    }
    pub fn ipttl(&self) -> &u8 {
        &self.ipttl
    }
    pub fn ipproto(&self) -> &u8 {
        &self.ipproto
    }
    pub fn version(&self) -> &u8 {
        &self.version
    }
    pub fn vrid(&self) -> &u8 {
        &self.vrid
    }
    pub fn prio(&self) -> &u8 {
        &self.prio
    }
    pub fn addrcount(&self) -> &u8 {
        &self.addrcount
    }
    // safer getter for addrcount, with checks for valid frame size
    pub fn s_addrcount(&self, framesize: usize) -> u8 {
        // make sure the address count matches the frame size,
        // a valid packet with one address should equal 60 bytes
        if framesize != 56 + (self.addrcount * 4) as usize {
            return 0u8;
        }
        self.addrcount
    }
    pub fn authtype(&self) -> &u8 {
        &self.authtype
    }
    pub fn adverint(&self) -> &u8 {
        &self.adverint
    }
    pub fn checksum(&self) -> &u16 {
        &self.checksum
    }
    // gen_advert() method
    // generate a VRRPv2 ADVERTISEMENT packet
    pub fn gen_advert(vr: &RwLockWriteGuard<'_, VirtualRouter>) -> VRRPpkt {
        // Ethernet frame headers:
        // dst multicast MAC address for 224.0.0.18
        let dst_mac = ETHER_VRRP_V2_DST_MAC;
        // generate source MAC address from VID
        let mut src_mac = ETHER_VRRP_V2_SRC_MAC;
        src_mac[5] = vr.parameters.vrid();
        // ipv4 ethertype
        let ethertype = ETHER_P_IP.to_be();

        // IPv4 headers:
        let ipver = IP_V4_VERSION;
        // dscp (CS6)
        let ipdscp = IP_DSCP_CS6;
        // lowest total packet length (header+data)
        let iplength = 40u16.to_be();
        // identification and flags fields to zeros
        let ipident = 0x0000;
        let ipflags = 0x0000;
        // TTL must be set to 255
        let ipttl = IP_TTL_VRRP_MINTTL;
        // VRRPv2 is IP Proto 112
        let ipproto = IP_UPPER_PROTO_VRRP;
        // internet checksum (set to all zeros)
        let ipchecksum = 0x0000;
        // source packet from interface 'primary' ip address
        let ipsrc = vr.parameters.primary_ip();
        // VRRPv2 multicast group
        let ipdst = VRRP_V2_IP_MCAST_DST;

        // VRRPv2 ADVERTISEMENT:
        // version = 0x2
        // type = 0x1 (ADVERTISEMENT)
        let version = VRRP_V2_ADVERT_VERSION_TYPE;
        // virtual router id
        let vrid = vr.parameters.vrid();
        let prio = vr.parameters.prio();
        let addrcount = vr.parameters.addrcount();
        let authtype = vr.parameters.authtype();
        let adverint = vr.parameters.adverint();
        // generate checksum on VRRP message
        let checksum = 0;

        // return the built VRRP ADVERTISEMENT packet
        VRRPpkt {
            dst_mac,
            src_mac,
            ethertype,
            ipver,
            ipdscp,
            iplength,
            ipident,
            ipflags,
            ipttl,
            ipproto,
            ipchecksum,
            ipsrc,
            ipdst,
            version,
            vrid,
            prio,
            addrcount,
            authtype,
            adverint,
            checksum,
        }
    }
}

// send_advertisement() function
/// Send a VRRP ADVERTISEMENT message
pub fn send_advertisement(
    sockfd: i32,
    vr: &RwLockWriteGuard<'_, VirtualRouter>,
    debug: &Verbose,
) -> io::Result<()> {
    // generate initial VRRP ADVERTISEMENT frame/packet
    let advert = VRRPpkt::gen_advert(vr);

    // build static frame slice
    let static_frame = unsafe { as_u8_slice(&advert) };

    // initialize frame_vec vector and push static frame into it
    let mut frame: Vec<u8> = Vec::new();
    for b in static_frame {
        frame.push(*b);
    }

    // set and push the VIP to the ipaddrs
    let vip = vr.parameters.vip();
    for i in 0..4 {
        frame.push(vip[i]);
    }

    // check if rfc3768 compatibility flag is false
    if !vr.parameters.rfc3768()  {
        // extend the frame with the variable-length list of local IP addresses
        for addr in vr.parameters.ipaddrs() {
            for i in 0..4 {
                frame.push(addr[i]);
            }
        }
    }

    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        format!("debug(packet): sending advertisement frame -> {:?}", frame),
    );

    // add authentication data
    for b in gen_auth_data(vr.parameters.authsecret()) {
        frame.push(b);
    }

    // generate VRRP checksum (vrrp checksum is at offset 34+6 bytes)
    let vrrp_checksum =
        checksums::one_complement_sum(&frame[VRRP_V2_FRAME_OFFSET..], Option::Some(6));
    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        format!("debug(packet): VRRP checksum is {:#X}", vrrp_checksum),
    );
    // set vrrp's checksum field
    frame[VRRP_V2_FRAME_OFFSET + 6] = vrrp_checksum.to_be() as u8;
    frame[VRRP_V2_FRAME_OFFSET + 6 + 1] = vrrp_checksum as u8;

    // generate IP checksum (ip checksum is at offset 14+10 bytes)
    let ip_checksum = checksums::one_complement_sum(&frame[IP_FRAME_OFFSET..], Option::Some(10));
    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        format!("debug(packet): IP checksum is {:#X}", ip_checksum),
    );

    // set ip checksum field (offset 34)
    frame[IP_FRAME_OFFSET + 10] = ip_checksum.to_be() as u8;
    frame[IP_FRAME_OFFSET + 10 + 1] = ip_checksum as u8;

    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        format!(
            "debug(packet): frame is {} bytes long",
            frame.len() - ETHER_FRAME_SIZE
        ),
    );
    // set length of ip packet (offset 16)
    // the length of ip header + data = frame size - ethernet frame
    let frame_size = frame.len() - ETHER_FRAME_SIZE;
    frame[IP_FRAME_OFFSET + 2] = frame_size.to_be() as u8;
    frame[IP_FRAME_OFFSET + 2 + 1] = frame_size as u8;

    // sockaddr_ll (man 7 packet)
    let mut sa = sockaddr_ll {
        sll_family: AF_PACKET as u16,
        sll_protocol: ETHER_P_IP.to_be(),
        sll_ifindex: vr.parameters.ifindex(),
        sll_hatype: 0,
        sll_pkttype: 0,
        sll_halen: 0,
        sll_addr: [0; 8],
    };

    unsafe {
        // unsafe call to sendto()
        let ptr_sockaddr = mem::transmute::<*mut sockaddr_ll, *mut sockaddr>(&mut sa);
        match sendto(
            sockfd,
            &mut frame[..] as *mut _ as *const c_void,
            mem::size_of_val(&frame[..]),
            0,
            ptr_sockaddr,
            mem::size_of_val(&sa) as u32,
        ) {
            -1 => Err(io::Error::last_os_error()),
            _ => {
                // println!(
                //     "debug: sucessfully sent {} bytes frame {:?}",
                //     frame.len(),
                //     frame
                // );
                Ok(())
            }
        }
    }
}

// as_u8_slice() unsafe function
/// transform type T as slice of u8
unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

// gen_auth_data() function
/// generate authentication data
fn gen_auth_data(secret: &Option<String>) -> Vec<u8> {
    match secret {
        // Auth Type 1 (RFC2338 backward-compatible)
        Some(s) => {
            let secpad = format!("{:\0<8}", s);
            secpad.into_bytes()
        }
        // Auth Type 0 (null authentication)
        None => {
            let mut auth_vec: Vec<u8> = Vec::new();
            for _ in 0..8 {
                auth_vec.push(0);
            }
            auth_vec
        }
    }
}
