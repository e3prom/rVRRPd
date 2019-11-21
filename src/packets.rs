//! packets handling module
//! This module includes the various packets formats and related functions such as internet checksums calculation function.

// constants
use crate::constants::*;

// virtual router
use crate::VirtualRouter;

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
        if framesize != 56 + (self.addrcount as usize * 4) as usize {
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
    pub fn gen_advert(vr: &VirtualRouter) -> VRRPpkt {
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

// as_u8_slice() unsafe function
/// transform type T as slice of u8
pub unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}
