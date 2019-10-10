//! FreeBSD network support

// libc
use libc::{IF_NAMESIZE, c_ulong, c_uint, c_int, c_short, c_void, sockaddr, ioctl};

// std
use std::io;
use std::ffi::CString;

// FreeBSD constants
use crate::os::freebsd::constants::*;

// IfReq Structure
#[repr(C)]
struct IfReq {
    ifr_name: [u8; IF_NAMESIZE],    // interface name
    ifru_addr: sockaddr,            // address
    ifru_dstaddr: sockaddr,         // remote ptp endpoint
    ifru_broadaddr: sockaddr,       // broadcast address
    ifru_buffer: ifreq_buffer,      // user supplied buffer with length
    ifru_flags: [c_short; 2],       // flags (high,low)
    ifru_index: c_short,            // interface index
    ifru_metric: c_int,             // metric
    ifru_mtu: c_int,                // maximum transmission unit
    ifru_phys: c_int,               // physical wire
    ifru_media: c_int,              // physical media
    ifru_data: c_int,               // interface data (replaced caddr_t with c_int)
    ifru_cap: [c_int; 2],           // capabilities (requested/current)
}

// ifreq_buffer Structure
struct ifreq_buffer {
    length: c_int,
    buffer: *const c_void,
} 

// set_ip_address() function
/// Set IPv4 Address on given interface
pub fn set_ip_address(fd: i32, ifname: &CString, ip: [u8; 4], netmask: [u8; 4]) -> io::Result<()> {
    // create a slice of mutable reference to array of 16 u8
    let ifname_slice = &mut [0u8; 16];
    // for every bytes/character in name of type Cstring, insert it into the above slice.
    for (i, b) in ifname.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b;
    }
    // check interface name size
    if ifname_slice.len() > IF_NAMESIZE {
        panic!("Interface name is longer than {}", IF_NAMESIZE - 1);
    }

    // create IP address slice
    let ip_addr_slice = &mut [0u8; 14];
    for (i, b) in ip.iter().enumerate() {
        ip_addr_slice[i] = *b;
    }

    // create IP netmask slice
    let ip_netmask_slice = &mut [0u8; 14];
    for (i, b) in netmask.iter().enumerate() {
        ip_netmask_slice[i] = *b;
    }

    // construct IfReq structure
    let mut ifaddr = IfReq {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            buf.clone_from_slice(ifname_slice); 
            buf
        },
        ifru_addr: sockaddr {
            sa_family: 0,
            sa_data: [0i8; 14],
            sa_len: 0,
        },
        ifru_dstaddr: sockaddr {
            sa_family: 0,
            sa_data: [0i8; 14],
            sa_len: 0,
        },
        ifru_broadaddr: sockaddr {
            sa_family: 0,
            sa_data: [0i8; 14],
            sa_len: 0,
        },
        ifru_buffer: ifreq_buffer {
            length: 0,
            buffer: {
                let buf = [0u8; 64]; 
                &buf as *const _ as *const c_void
            } 
        },
        ifru_flags: [0i16; 2],
        ifru_index: 0,
        ifru_metric: 0,
        ifru_mtu: 1500,
        ifru_phys: 0,
        ifru_media: 0,
        ifru_data: 0,
        ifru_cap: [0i32; 2],
    };

    // see man 4 netintro
    // ioctl - set interface's IP address
    let res = unsafe { ioctl(fd, SIOCAIFADDR, &mut ifaddr) };
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    // // ioctl - set interface's netmask
    // let res = unsafe { ioctl(fd, SIOCSIFNETMASK, &mut ifnetmask) };
    // if res < 0 {
    //     return Err(io::Error::last_os_error());
    // }
    
    Ok(())
} 
