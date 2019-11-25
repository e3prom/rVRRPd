//! Linux Socket Filter module

// libc
use libc::c_void;

// SockFilter structure
#[repr(C)]
pub struct SockFilter {
    code: u16, // Filter code
    jt: u8,    // Jump true
    jf: u8,    // Jump false
    k: u32,    // Generic multiuse field
}

// SockFilter implementation
impl SockFilter {
    // new_vrrpv2_gid() method
    //
    // BPF Filter - VRRPv2 Advertisement Packets:
    // ldh      [12]
    // jne      #0x800, drop
    // ldb      [23]
    // jneq     #0x70, drop
    // ldb      [34]
    // jneq     #0x21, drop
    // ldb      [35]
    // jneq     #0x1, drop
    // ret      #-1
    // drop:    ret #0
    //
    // BPF Bytecode:
    // { 0x28,  0,  0, 0x0000000c },
    // { 0x15,  0,  7, 0x00000800 },
    // { 0x30,  0,  0, 0x00000017 },
    // { 0x15,  0,  5, 0x00000070 },
    // { 0x30,  0,  0, 0x00000022 },
    // { 0x15,  0,  3, 0x00000021 },
    // { 0x30,  0,  0, 0x00000023 },
    // { 0x15,  0,  1, 0x00000001 },
    // { 0x06,  0,  0, 0xffffffff },
    // { 0x06,  0,  0, 0000000000 },
    //
    pub fn new_vrrpv2_gid(gid: u8) -> [SockFilter; 10] {
        let filter: [SockFilter; 10] = [
            SockFilter {
                // 001
                code: 0x28,
                jt: 0x0,
                jf: 0x0,
                k: 0x0000000c,
            },
            SockFilter {
                // 002
                code: 0x15,
                jt: 0x0,
                jf: 0x7,
                k: 0x00000800,
            },
            SockFilter {
                // 003
                code: 0x30,
                jt: 0x0,
                jf: 0x0,
                k: 0x00000017,
            },
            SockFilter {
                // 004
                code: 0x15,
                jt: 0x0,
                jf: 0x5,
                k: 0x00000070,
            },
            SockFilter {
                // 005
                code: 0x30,
                jt: 0x0,
                jf: 0x0,
                k: 0x00000022,
            },
            SockFilter {
                // 006
                code: 0x15,
                jt: 0x0,
                jf: 0x0,
                k: 0x00000021,
            },
            SockFilter {
                // 007
                code: 0x30,
                jt: 0x0,
                jf: 0x0,
                k: 0x00000023,
            },
            SockFilter {
                // 008
                code: 0x15,
                jt: 0x0,
                jf: 0x1,
                k: gid as u32, // replace by the group id
            },
            SockFilter {
                // 009
                code: 0x06,
                jt: 0x0,
                jf: 0x0,
                k: 0xffffffff,
            },
            SockFilter {
                // 010
                code: 0x06,
                jt: 0x0,
                jf: 0x0,
                k: 0000000000,
            },
        ];
        filter
    }
}

// SockFprog structure
#[repr(C)]
pub struct SockFprog {
    len: u16,              // Number of filter blocks (size of array)
    filter: *const c_void, // Pointer to an array of SockFilter
}

// SockFprog implementation
impl SockFprog {
    // build_fprog_vrrpv2_gid() method
    pub fn build_fprog_vrrpv2_gid(filter: &[SockFilter; 10]) -> SockFprog {
        let fprog = SockFprog {
            filter: filter.as_ptr() as *const c_void,
            len: filter.len() as u16,
        };
        fprog
    }
}
