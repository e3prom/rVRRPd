// FreeBSD standard C library support

// std, libc
use std::io;
use libc::{read, c_void};

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
                println!("DEBUG: error while reading BPF buffer on fd {}, buffer length {}", bpf_fd, buf.len());
                return Err(io::Error::last_os_error());
            }
            len => len,
        }
    }

    // return the length of read buffer
    Ok(len)
}