//! FreeBSD network support

// libc
use libc::{c_int, c_short, c_uchar, ioctl, AF_INET, IF_NAMESIZE};

// std
use std::ffi::CString;
use std::io;

// FreeBSD constants
use crate::os::freebsd::constants::*;

// operating systems drivers
use crate::os::drivers::Operation;

// IfAliasReq Structure
#[repr(C)]
struct IfAliasReq {
    ifr_name: [u8; IF_NAMESIZE],     // interface name
    ifra_addr: int_sockaddr_in,      // IPv4 address
    ifra_broadaddr: int_sockaddr_in, // destination address
    ifra_mask: int_sockaddr_in,      // netmask
    ifra_vhid: c_int,
}

// int_sockaddr alias
#[repr(C)]
struct int_sockaddr_in {
    sin_len: c_uchar,
    sin_family: c_uchar,
    sin_port: c_short,
    sin_addr: [c_uchar; 12],
}

// set_ip_address() function
/// Set IPv4 Address on given interface
pub fn set_ip_address(
    _fd: i32,
    ifname: &CString,
    ip: [u8; 4],
    netmask: [u8; 4],
    op: Operation,
) -> io::Result<()> {
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
    let ip_addr_slice = &mut [0u8; 12];
    for (i, b) in ip.iter().enumerate() {
        ip_addr_slice[i] = *b;
    }

    // create IP netmask slice
    let ip_netmask_slice = &mut [0u8; 12];
    for (i, b) in netmask.iter().enumerate() {
        ip_netmask_slice[i] = *b;
    }

    let mut ifaddr = IfAliasReq {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            buf.clone_from_slice(ifname_slice);
            buf
        },
        ifra_addr: int_sockaddr_in {
            sin_len: 16,
            sin_family: AF_INET as u8,
            sin_port: 0,
            sin_addr: {
                let mut data = [0u8; 12];
                data.clone_from_slice(ip_addr_slice);
                data
            },
        },
        ifra_broadaddr: int_sockaddr_in {
            sin_len: 0,
            sin_family: 0,
            sin_port: 0,
            sin_addr: [0u8; 12],
        },
        ifra_mask: int_sockaddr_in {
            sin_len: 16,
            sin_family: AF_INET as u8,
            sin_port: 0,
            sin_addr: {
                let mut data = [0u8; 12];
                data.clone_from_slice(ip_netmask_slice);
                data
            },
        },
        ifra_vhid: 0,
    };

    // open new socket for below ioctl
    let fd = match unsafe { libc::socket(libc::PF_INET, libc::SOCK_DGRAM, 0) } {
        -1 => return Err(io::Error::last_os_error()),
        fd => fd,
    };

    // match given operation on IP address
    // see man 4 netintro
    match op {
        Operation::Add => {
            // ioctl - set interface's IP address
            let res = unsafe { ioctl(fd, SIOCAIFADDR, &mut ifaddr) };
            if res < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Operation::Rem => {
            // ioctl - remove interface's IP address
            let res = unsafe { ioctl(fd, SIOCDIFADDR, &mut ifaddr) };
            if res < 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }

    Ok(())
}
