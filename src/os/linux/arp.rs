//! Linux Address Resolution Protocol (ARP) module
//! This module provides ARP related functions.

// std
use std::io;

// libc
use libc::{socket, AF_PACKET, ETH_P_ARP, SOCK_RAW};

/// Address Resolution Protocol (ARP) Structure
#[repr(C)]
pub struct ARPframe {
    // Ethernet Header
    pub dst_mac: [u8; 6], // destination MAC address
    pub src_mac: [u8; 6], // source MAC address
    pub ethertype: u16,   // ether type

    // ARP
    pub hardware_type: u16,         // network link type (0x1=ethernet)
    pub protocol_type: u16,         // upper-layer protocol for resolution
    pub hw_addr_len: u8,            // length of hardware address (bytes)
    pub proto_addr_len: u8,         // upper-layer protocol address length
    pub opcode: u16,                // operation (0x1=request, 0x2=reply)
    pub sender_hw_addr: [u8; 6],    // sender hardware address
    pub sender_proto_addr: [u8; 4], // internetwork address of sender
    pub target_hw_addr: [u8; 6],    // hardware address of target
    pub target_proto_addr: [u8; 4], // internetwork address of target
}

// open_raw_socket_arp() function
/// Open raw socket
pub fn open_raw_socket_arp() -> io::Result<i32> {
    unsafe {
        // man 2 socket
        // returns a file descriptor or -1 if error.
        match socket(AF_PACKET, SOCK_RAW, ETH_P_ARP.to_be() as i32) {
            -1 => Err(io::Error::last_os_error()),
            fd => Ok(fd),
        }
    }
}
