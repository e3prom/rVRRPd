//! FreeBSD Berkeley Packet Filter (BPF) module

// std
use std::io;
use std::convert::TryInto;

// libc
use libc::{IF_NAMESIZE};

// ffi
use std::ffi::{CString};

// FreeBSD constants
use crate::os::freebsd::constants::*;

// Ifreq redifinition (stripped)
#[repr(C)]
struct IfreqS {
    ifr_name: [u8; IF_NAMESIZE],
}

// BPF system structures
// bpf_ts structure
// https://github.com/freebsd/freebsd/blob/master/sys/net/bpf.h:202
#[repr(C)]
pub struct bpf_ts {
    pub bt_sec: i64,
    pub bt_frac: u64,
}

// bpf_xhdr structure
#[repr(C)]
pub struct bpf_xhdr {
    pub bh_tstamp: bpf_ts,  // timestamp
    pub bh_caplen: u32,     // length of captured pattern
    pub bh_datalen: u32,    // length of packet
    pub bh_hdrlen: u16,     // length of this structure + alignment padding
}

// bpf_open_device() function
//
/// Open a BPF device and return the file descriptor if successful
pub fn bpf_open_device() -> io::Result<(i32)> {
    // try /dev/bpf
    let bpf_dev = CString::new("/dev/bpf").unwrap();
    let bpf_dev_slice = &mut [0i8; 10];
    for (i, b) in bpf_dev.as_bytes_with_nul().iter().enumerate() {
        bpf_dev_slice[i] = (*b).try_into().unwrap();
    }
    let mut buf = [0i8; 10];
    buf.clone_from_slice(bpf_dev_slice);

    // open /dev/bpf device
    println!("DEBUG: opening /dev/bpf device");
    let res = unsafe {libc::open(&buf as *const i8, libc::O_RDWR)};
    if res >= 0 {
        return Ok(res);
    }

    // if above failed, try /dev/bpfX devices
    for i in 0..99 {
        // create bpf device name slice
        let bpf_fmtstr = format!("/dev/bpf{}", i);
        let bpf_dev = CString::new(bpf_fmtstr).unwrap();
        let bpf_dev_slice = &mut [0i8; 11];
        for (i, b) in bpf_dev.as_bytes_with_nul().iter().enumerate() {
            bpf_dev_slice[i] = (*b).try_into().unwrap();
        }
        // create bpf device name buffer
        let mut buf = [0i8; 11];
        buf.clone_from_slice(bpf_dev_slice);

        // open bpf device
        println!("DEBUG: opening device /dev/bpf{}", i);
        let res = unsafe {libc::open(&buf as *const i8, libc::O_RDWR)};

        // check returned value
        // if negative, an error occured, continue
        // if positive, return the file descriptor
        if res >= 0 {
            return Ok(res);
        }
    }

    // if all BPF devices are exhausted
    println!("cannot find an available BPF device");
    return Err(io::Error::last_os_error());
}

// bpf_bind_device() function
//
/// Bind BPF device to a physical interface 
pub fn bpf_bind_device(bpf_fd: i32, interface: &CString) -> io::Result<()> {
    let ifname_slice = &mut [0u8; IF_NAMESIZE];
    for (i, b) in interface.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b; 
    }

    // create Ifreq structure
    let ifbound = IfreqS {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            buf.clone_from_slice(ifname_slice); 
            buf
        }
    };

    // ioctl
    println!("DEBUG: binding BPF device with fd {} to interface {:?}", bpf_fd, interface);
    match unsafe { libc::ioctl(bpf_fd, BIOCSETIF, &ifbound) } {
        r if r >= 0 => Ok(()),
        e => {
            println!("error while binding BPF device, fd {}, error no: {}", bpf_fd, e);
            return Err(io::Error::last_os_error());
        }
    }
}

// bpf_setup_buf() function
//
/// Setup BPF device buffer and features
/// Return size of BPF buffer after setup
pub fn bpf_setup_buf(bpf_fd: i32, pkt_buf: &mut [u8]) -> io::Result<(usize)> {
    // initialize local buf_len with current buffer size
    let buf_len = pkt_buf.len();

    if buf_len == 0 { 
        // get buffer length (ioctl)
        // actually ignoring returned value
        match unsafe { libc::ioctl(bpf_fd, BIOCGBLEN, &buf_len)} {
            e if e < 0 => {
                println!("error while getting buffer length on BPF device, fd {}, error no: {}", bpf_fd, e);
                return Err(io::Error::last_os_error());
            }
            s => {
                println!("DEBUG: required buffer length for BPF device, fd {}, is: {} bytes", bpf_fd, s);
            }
        };
    } else {   
        // set buffer length (ioctl)
        match unsafe { libc::ioctl(bpf_fd, BIOCSBLEN, &buf_len)} {
            e if e < 0 => {
                println!("error while setting buffer length on BPF device, fd {}, error no: {}", bpf_fd, e);
                return Err(io::Error::last_os_error());
            }
            _ => {
                println!("DEBUG: buffer length for BPF device, fd {} set", bpf_fd);
            }
        };
    } 

    // activate immediate mode (ioctl)
    match unsafe { libc::ioctl(bpf_fd, BIOCIMMEDIATE, &buf_len) } {
        e if e < 0 => {
            println!("error while setting immediate mode on BPF device, fd {}, error no: {}", bpf_fd, e);
            return Err(io::Error::last_os_error());
        }
        _ => {
            println!("DEBUG: immediate mode set on BPF device, fd {}", bpf_fd);
        }
    };

    // set the header complete flag to one
    let flag = 1;
    match unsafe { libc::ioctl(bpf_fd, BIOCSHDRCMPLT, &flag) } {
        e if e < 0 => {
            println!("error while setting ({}) header complete flag on BPF device, fd {}, error no: {}", flag, bpf_fd, e);
            return Err(io::Error::last_os_error());
        }
        _ => {
            println!("DEBUG: header complete flag set to {} on BPF device, fd {}", flag, bpf_fd);
        }
    };

    // return Ok(buf_len) if everything went successful
    Ok(buf_len as usize)
}

// bpf_set_promisc() function
//
/// Set interface bound to the BPF's fd in promiscuous mode
pub fn bpf_set_promisc(bpf_fd: i32) -> io::Result<()> {
    // set interface in promiscuous mode
    match unsafe { libc::ioctl(bpf_fd, BIOCPROMISC.into(), 0) } {
        e if e < 0 => {
            println!("error while setting promiscuous mode on BPF device, fd {}, error no: {}", bpf_fd, e);
            return Err(io::Error::last_os_error());
        }
        _ => {
            println!("DEBUG: promiscuous mode set on BPF device, fd {}", bpf_fd);
            Ok(())
        }
    }
} 

// bpf_wordalign() function
//
/// Align the BPF buffer to the next frame given capured size
/// Reference: pnet's source/src/bindings/bpf.rs
pub fn bpf_wordalign(s: isize) -> isize {
    let bpf_alignement = BPF_ALIGNMENT as isize;
    let one = 1;

    (s + (bpf_alignement - one)) & !(bpf_alignement - one)
}
