//! FreeBSD specific constants

// libc
use libc::{c_ulong, c_uint, c_int};

// sockios, ioctls
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
pub const SIOCDIFADDR: c_ulong = 0x80206919;
pub const SIOCGIFCONF: c_ulong = 0xc0106924;
pub const SIOCGIFFLAGS: c_ulong = 0xc0206911;
pub const SIOCSIFFLAGS: c_ulong = 0x80206910;
pub const SIOCGIFDSTADDR: c_ulong = 0xc0206922;
pub const SIOCSIFDSTADDR: c_ulong = 0x8020690e;
pub const SIOCGIFBRDADDR: c_ulong = 0xc0206923;
pub const SIOCSIFBRDADDR: c_ulong = 0x80206913;
pub const SIOCGIFNETMASK: c_ulong = 0xc0206925;
pub const SIOCSIFNETMASK: c_ulong = 0x80206916;
pub const SIOCGIFMETRIC: c_ulong = 0xc0206917;
pub const SIOCSIFMETRIC: c_ulong = 0x80206918;
pub const SIOCGIFMTU: c_ulong = 0xc0206933;
pub const SIOCSIFMTU: c_ulong = 0x80206934;
pub const SIOCADDMULTI: c_ulong = 0x80206931;
pub const SIOCDELMULTI: c_ulong = 0x80206932;

// Berkeley Packet Filter
pub const BPF_ALIGNMENT: c_int = 8;
