//! Linux Address Resolution Protocol (ARP) module
//! This module provides ARP related functions.

// constants
use crate::constants::*;

// channels and threads
use std::sync::RwLockWriteGuard;

// virtual router
use crate::VirtualRouter;

// std
use std::io;
use std::mem;

// libc
use libc::{c_void, sendto, sockaddr, sockaddr_ll, socket, AF_PACKET, ETH_P_ARP, SOCK_RAW};

/// Address Resolution Protocol (ARP) Structure
#[repr(C)]
struct ARPframe {
    // Ethernet Header
    dst_mac: [u8; 6], // destination MAC address
    src_mac: [u8; 6], // source MAC address
    ethertype: u16,   // ether type

    // ARP
    hardware_type: u16,         // network link type (0x1=ethernet)
    protocol_type: u16,         // upper-layer protocol for resolution
    hw_addr_len: u8,            // length of hardware address (bytes)
    proto_addr_len: u8,         // upper-layer protocol address length
    opcode: u16,                // operation (0x1=request, 0x2=reply)
    sender_hw_addr: [u8; 6],    // sender hardware address
    sender_proto_addr: [u8; 4], // internetwork address of sender
    target_hw_addr: [u8; 6],    // hardware address of target
    target_proto_addr: [u8; 4], // internetwork address of target
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

// broadcast_gratuitious_arp() function
/// Broadcast Gratuitious ARP requests
pub fn broadcast_gratuitious_arp(
    sockfd: i32,
    vr: &RwLockWriteGuard<'_, VirtualRouter>,
) -> io::Result<()> {
    // build gratuitious ARP request
    let mut arpframe = ARPframe {
        dst_mac: ETHER_ARP_DST_MAC,
        src_mac: ETHER_VRRP_V2_SRC_MAC,
        ethertype: ETHER_P_ARP.to_be(),

        hardware_type: ARP_HW_TYPE.to_be(),
        protocol_type: ETHER_P_IP.to_be(),
        hw_addr_len: 6,
        proto_addr_len: 4,
        opcode: ARP_OP_REQUEST.to_be(),
        sender_hw_addr: ETHER_VRRP_V2_SRC_MAC,
        sender_proto_addr: vr.parameters.vip(),
        target_hw_addr: [0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
        target_proto_addr: [255, 255, 255, 255],
    };

    // set VRID on source MAC addresses
    arpframe.src_mac[5] = vr.parameters.vrid();
    arpframe.sender_hw_addr[5] = vr.parameters.vrid();

    // sockaddr_ll (man 7 packet)
    let mut sa = sockaddr_ll {
        sll_family: AF_PACKET as u16,
        sll_protocol: ETHER_P_ARP.to_be(),
        sll_ifindex: vr.parameters.ifindex(),
        sll_hatype: 0,
        sll_pkttype: 0,
        sll_halen: 0,
        sll_addr: [0; 8],
    };

    unsafe {
        let ptr_sockaddr = mem::transmute::<*mut sockaddr_ll, *mut sockaddr>(&mut sa);
        match sendto(
            sockfd,
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
