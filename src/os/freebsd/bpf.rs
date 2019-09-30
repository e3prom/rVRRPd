//! FreeBSD Berkeley Packet Filter (BPF) module

// std
use std::io;
use std::convert::TryInto;

// ffi
use std::ffi::{CString};

pub fn open_bpf_device() -> io::Result<(i32)> {
    // find an available BPF device
    for i in 0..99 {
        // print information
        println!("Opening device /dev/bpf{}", i);

        // build device name
        // let mut buf: [i8; 11] = [0; 11];
        // let bpf_fmtstr = "/dev/bpf%i";
        // let bpf_cstr = CString::new(bpf_fmtstr).unwrap();
        // libc::sprintf(buf as *mut i8, bpf_cstr as *const i8, i);

        // create bpf device name slice
        let bpf_fmtstr = format!("/dev/bpf{}", i);
        let bpf_dev = CString::new(bpf_fmtstr).unwrap();
        let mut bpf_dev_slice = &mut [0i8; 11];
        for (i, b) in bpf_dev.as_bytes_with_nul().iter().enumerate() {
            bpf_dev_slice[i] = (*b).try_into().unwrap();
        }
        // create bpf device name buffer
        let mut buf = [0i8; 11];
        buf.clone_from_slice(bpf_dev_slice);

        // open BPF device
        println!("Opening device /dev/bpf{}", i);
        let res = unsafe {libc::open(&buf as *const i8, libc::O_RDWR)};

        // check returned value
        // if negative, an error occured, continue
        // if positive, return the file descriptor
        if res >= 0 {
            return Ok(res);
        }
    }

    // if all BPF devices are exhausted
    println!("Error, cannot find an available BPF device");
    return Err(io::Error::last_os_error());
}