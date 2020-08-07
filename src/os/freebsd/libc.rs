//! FreeBSD standard C library support
use crate::*;

// std, libc
use libc::{c_void, read, write};
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
                eprintln!(
                    "error while reading BPF buffer on fd {}, buffer length {}",
                    bpf_fd,
                    buf.len()
                );
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
pub fn raw_sendto(fd: i32, _ifindex: i32, frame: &mut Vec<u8>, debug: &Verbose) -> io::Result<()> {
    unsafe {
        // unsafe call to write()
        match write(
            fd,
            &mut frame[..] as *mut _ as *const c_void,
            mem::size_of_val(&frame[..]),
        ) {
            -1 => Err(io::Error::last_os_error()),
            _ => {
                print_debug(
                    debug,
                    DEBUG_LEVEL_MEDIUM,
                    DEBUG_SRC_BPF,
                    format!("VRRPv2 frame successfully sent on BPF device, fd {}", fd),
                );
                Ok(())
            }
        }
    }
}
