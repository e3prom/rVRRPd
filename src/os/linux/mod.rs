//! Linux Operating System support
use crate::*;

// standard C library compatibility
pub mod libc;
// netdev support
pub mod netdev;
// libnl netlink support
pub mod libnl;
// Linux ARP support
pub mod arp;
// Linux Socket Filter support
pub mod filter;

// operating system drivers
use crate::os::drivers::Operation;

/// internal IP route structure
pub struct IntIpRoute<'a> {
    pub sockfd: i32,
    pub ifname: String,
    pub route: [u8; 4],
    pub rtmask: [u8; 4],
    pub gw: [u8; 4],
    pub metric: i16,
    pub mtu: u64,
    pub op: &'a Operation,
    pub debug: &'a Verbose,
}
