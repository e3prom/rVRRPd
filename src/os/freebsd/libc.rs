// FreeBSD standard C library support

// std, libc
use std::io;
use libc::{read, c_void};

// read_bpf_buf() function
//
/// Receive IP Packet
pub fn read_bpf_buf(bpf_fd: i32, buf: &mut [u8] ) -> io::Result<usize> {
    // declare len
    let len: isize;

    // read from BPF device (unsafe)
    unsafe {
        let len = read(bpf_fd, buf.as_ptr() as *mut c_void, buf.len());
    } 

    // debug
    println!("DEBUG Frame/Packet: {:?}", buf);

    // return the length of read buffer
    Ok(len as usize)
}