//! FreeBSD Berkeley Packet Filter (BPF) module

// std
use std::io;
use std::convert::TryInto;

// libc
use libc::{c_char, sockaddr, IF_NAMESIZE, c_ulong, c_uint};

// ffi
use std::ffi::{CString};

// Ifreq redifinition
#[repr(C)]
struct Ifreq {
    ifr_name: [u8; IF_NAMESIZE],
    // ifru_addr: sockaddr,
}

// BPF constants (until added to rust's libc)
// https://github.com/equal-l2/libc/blob/cf1a3e10fa95a33d8f987e29b8d91e0db91c9cb0/src/unix/bsd/mod.rs
const BIOCGBLEN: c_ulong = 0x40044266;
const BIOCSBLEN: c_ulong = 0xc0044266;
const BIOCFLUSH: c_uint = 0x20004268;
const BIOCPROMISC: c_uint = 0x20004269;
const BIOCGDLT: c_ulong = 0x4004426a;
const BIOCGETIF: c_ulong = 0x4020426b;
const BIOCSETIF: c_ulong = 0x8020426c;
const BIOCGSTATS: c_ulong = 0x4008426f;
const BIOCIMMEDIATE: c_ulong = 0x80044270;
const BIOCVERSION: c_ulong = 0x40044271;
const BIOCGRSIG: c_ulong = 0x40044272;
const BIOCSRSIG: c_ulong = 0x80044273;
const BIOCGHDRCMPLT: c_ulong = 0x40044274;
const BIOCSHDRCMPLT: c_ulong = 0x80044275;
const BIOCGSEESENT: c_ulong  = 0x40044276;
const BIOCSSEESENT: c_ulong  = 0x80044277;
const BIOCSDLT: c_ulong = 0x80044278;
const SIOCGIFADDR: c_ulong = 0xc0206921;

// bpf_open_device() function
//
/// Open BPF device and return file descriptor if successful
pub fn bpf_open_device() -> io::Result<(i32)> {
    // find an available BPF device
    for i in 0..99 {
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
        println!("DEBUG: opening device /dev/bpf{}", i);
        let res = unsafe {libc::open(&buf as *const i8, libc::O_RDWR)};

        // check returned value
        // if negative, an error occured, continue
        // if positive, return the file descriptor
        //println!("DEBUG: open() for /dev/bpf{}, returned: {}", i, res);
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
        }
        // ifru_addr: sockaddr {
        //     sa_family: 0,
        //     sa_data: [0; 14],
        //     sa_len: 0,
        // },
    };

    // ioctl
    println!("DEBUG: binding BPF device with fd {} to interface {:?}", bpf_fd, interface);
    match unsafe { libc::ioctl(bpf_fd, BIOCSETIF, &ifbound) } {
        r if r >= 0 => Ok(()),
        e => {
            println!("DEBUG: error while binding BPF device, fd {}, error no: {}", bpf_fd, e);
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
    let mut buf_len = pkt_buf.len();

    // activa1e immediate mode (ioctl)
    match unsafe { libc::ioctl(bpf_fd, BIOCIMMEDIATE, &buf_len) } {
        e if e < 0 => {
            println!("DEBUG: error while setting immediate mode on BPF device, fd {}, error no: {}", bpf_fd, e);
            return Err(io::Error::last_os_error());
        }
        _ => {
            println!("DEBUG: immediate mode set on BPF device, fd {}", bpf_fd);
        }
    };

    // set buffer length (ioctl)
    match unsafe { libc::ioctl(bpf_fd, BIOCGBLEN, &buf_len)} {
        e if e < 0 => {
            println!("DEBUG: error while setting buffer length on BPF device, fd {}, error no: {}", bpf_fd, e);
            return Err(io::Error::last_os_error());
        }
        _ => {
            println!("DEBUG: buffer length set on BPF device, fd {}, len {} ", bpf_fd, pkt_buf.len());
        }
    }

    // return Ok(buf_len) if everything went successful
    Ok(buf_len)
}
