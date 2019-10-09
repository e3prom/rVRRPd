// FreeBSD standard C library support

// std, libc
use libc::{read, c_void};
use std::io;
use std::mem;

// read_bpf_buf() function
//
/// Receive IP Packet
pub fn read_bpf_buf(bpf_fd: i32, buf: &mut [u8], buf_size: usize) -> io::Result<isize> {
    // declare len
    let len: isize;

    // read from BPF device (unsafe)
    unsafe {
        len = match read(bpf_fd, buf.as_ptr() as *mut c_void, buf_size) {
            -1 => {
                println!("error while reading BPF buffer on fd {}, buffer length {}", bpf_fd, buf.len());
                return Err(io::Error::last_os_error());
            }
            len => len,
        }
    }

    // return the length of read buffer
    Ok(len)
}

// raw_sendto() function
/// Send RAW frame/packet
fn raw_sendto(
    fd: i32,
    ifindex: i32,
    frame: &mut Vec<u8>,
) -> io::Result<()> {
    // sockaddr
    let mut sa = libc::sockaddr {
        sll_family: libc::AF_PACKET as u16,
        sll_protocol: ETHER_P_IP.to_be(),
        sll_ifindex: vr.parameters.ifindex(),
        sll_hatype: 0,
        sll_pkttype: 0,
        sll_halen: 0,
        sll_addr: [0; 8],
    };

    unsafe {
        // unsafe call to sendto()
        match libc::sendto(
            fd,
            &mut frame[..] as *mut _ as *const libc::c_void,
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
