//! FreeBSD specific constants
// libc
use libc::{c_ulong, c_uint, c_int};

// BPF and network related
pub const BIOCGBLEN: c_ulong = 0x40044266;
pub const BIOCSBLEN: c_ulong = 0xc0044266;
pub const BIOCFLUSH: c_uint = 0x20004268;
pub const BIOCPROMISC: c_uint = 0x20004269;
pub const BIOCGDLT: c_ulong = 0x4004426a;
pub const BIOCGETIF: c_ulong = 0x4020426b;
pub const BIOCSETIF: c_ulong = 0x8020426c;
pub const BIOCGSTATS: c_ulong = 0x4008426f;
pub const BIOCIMMEDIATE: c_ulong = 0x80044270;
pub const BIOCVERSION: c_ulong = 0x40044271;
pub const BIOCGRSIG: c_ulong = 0x40044272;
pub const BIOCSRSIG: c_ulong = 0x80044273;
pub const BIOCGHDRCMPLT: c_ulong = 0x40044274;
pub const BIOCSHDRCMPLT: c_ulong = 0x80044275;
pub const BIOCGSEESENT: c_ulong  = 0x40044276;
pub const BIOCSSEESENT: c_ulong  = 0x80044277;
pub const BIOCSDLT: c_ulong = 0x80044278;
pub const SIOCGIFADDR: c_ulong = 0xc0206921;
pub const SIOCAIFADDR: c_ulong = 0x8040691a;
pub const BPF_ALIGNMENT: c_int = 8;
