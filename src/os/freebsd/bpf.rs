//! FreeBSD Berkeley Packet Filter (BPF) module

// std
use std::io;
use std::convert::TryInto;

// libc
use libc::{c_char, sockaddr, IF_NAMESIZE};

// ffi
use std::ffi::{CString};

// Ifreq redifinition
struct Ifreq {
    ifr_name: [u8; IF_NAMESIZE],
    ifru_addr: sockaddr,
}

// bpf_open_device() function
//
/// Open BPF device and return file descriptor if successful
pub fn bpf_open_device() -> io::Result<(i32)> {
    // find an available BPF device
    for i in 0..99 {
        // print information
        println!("Opening device /dev/bpf{}", i);

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

// bpf_bind_device() function
//
/// Bind BPF device to a physical interface 
pub fn bpf_bind_device(bpf_fd: i32, interface: &CString) -> io::Result<()> {
    // // create physical interface cstring
    // let mut ifname = interface;
    // let ifname = interface.push_str("\0");

    let ifname_slice = &mut [0u8; IF_NAMESIZE];
    for (i, b) in interface.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b; 
    }

    // create Ifreq structure
    let ifbound = Ifreq {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            buf.clone_from_slice(ifname_slice); 
            buf
        },
        ifru_addr: sockaddr {
            sa_family: 0,
            sa_data: [0; 14], 
            sa_len: 0,
        },
    };

    // ioctl
    match unsafe { libc::ioctl(bpf_fd, BIOCSETIF, &ifbound) } {
        r if r > 0 => Ok(()),
        _ => Err(io::Error::last_os_error()),
    }
}

// bpf_setup_buf() function
//
/// Setup BPF device buffer and features
pub fn bpf_setup_buf(bpf_fd: i32) -> io::Result<()> {
    // create and initialize buffer_length
    let buf_len = 1;

    // activa1e immediate mode (ioctl)
    match unsafe { libc::ioctl(bpf_fd, BIOCIMMEDIATE, &buf_len) } {
        r if r < 0 => return Err(io::Error::last_os_error()),
        _ => {}
    }

    // set buffer length (ioctl)
    match unsafe { libc::ioctl(bpf_fd, BIOCGBLEN, &buf_len) } {
        r if r < 0 => return Err(io::Error::last_os_error()),
        _ => {}
    }

    // return Ok(()) if everything went successful
    Ok(())
}
