//! Linux standard C library compatibility
use crate::*;

// std, libc, ffi
use libc::{socket, AF_PACKET, SOCK_RAW};
use std::ffi::CString;
use std::io;
use std::mem;

// open_raw_socket_fd() function
/// Open a raw AF_PACKET socket for IPv4
pub fn open_raw_socket_fd() -> io::Result<i32> {
    unsafe {
        // man 2 socket
        // returns a file descriptor or -1 if error.
        match socket(AF_PACKET, SOCK_RAW, ETHER_P_IP.to_be() as i32) {
            -1 => Err(io::Error::last_os_error()),
            fd => Ok(fd),
        }
    }
}

// recv_ip_pkts() function
/// Receive IP packets
pub fn recv_ip_pkts(sockfd: i32, sockaddr: &mut sockaddr_ll, buf: &mut [u8]) -> io::Result<usize> {
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

// c_ifnametoindex() function
/// see 'man 3 if_nametoindex'
pub fn c_ifnametoindex(ifname: &String) -> io::Result<u32> {
    unsafe {
        let c_ifname = CString::new(ifname.clone()).unwrap();
        let r = libc::if_nametoindex(c_ifname.as_ptr());
        if r == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(r)
        }
    }
}

// raw_sendto() function
/// Send RAW frame/packet
pub fn raw_sendto(sockfd: i32, ifindex: i32, frame: &mut Vec<u8>) -> io::Result<()> {
    // sockaddr_ll (man 7 packet)
    let mut sa = libc::sockaddr_ll {
        sll_family: libc::AF_PACKET as u16,
        sll_protocol: ETHER_P_IP.to_be(),
        sll_ifindex: ifindex,
        sll_hatype: 0,
        sll_pkttype: 0,
        sll_halen: 0,
        sll_addr: [0; 8],
    };

    unsafe {
        // unsafe call to sendto()
        let ptr_sockaddr = mem::transmute::<*mut libc::sockaddr_ll, *mut libc::sockaddr>(&mut sa);
        match libc::sendto(
            sockfd,
            &mut frame[..] as *mut _ as *const c_void,
            mem::size_of_val(&frame[..]),
            0,
            ptr_sockaddr,
            mem::size_of_val(&sa) as u32,
        ) {
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }
}
