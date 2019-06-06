//! Constants module
//! This module regroups all the program's and protocols constants.

// Program Constants
pub const RVRRPD_DFLT_CFG_FILE: &str = "/etc/rvrrpd/rvrrpd.conf";
pub const RVRRPD_DFLT_PIDFILE: &str = "/var/run/rvrrpd.pid";
pub const RVRRPD_DFLT_WORKDIR: &str = "/tmp";
pub const RVRRPD_DFLT_LOGFILE: &str = "/var/log/rvrrpd.log";
pub const RVRRPD_DFLT_ELOGFILE: &str = "/var/log/rvrrpd-error.log";
pub const DEBUG_LEVEL_NONE: u8 = 0;
pub const DEBUG_LEVEL_LOW: u8 = 1;
pub const DEBUG_LEVEL_MEDIUM: u8 = 2;
pub const DEBUG_LEVEL_HIGH: u8 = 3;
pub const DEBUG_LEVEL_EXTENSIVE: u8 = 5;

// Ethernet Constants
pub const ETHER_P_IP: u16 = 0x0800; // IPv4 (/usr/include/linux/if_ether.h)
pub const ETHER_P_ARP: u16 = 0x0806;
pub const ETHER_VRRP_IPADDR_POS: usize = 42; // Position of the IP addresses variable-length field
pub const ETHER_VRRP_V2_SRC_MAC: [u8; 6] = [0x00, 0x00, 0x5e, 0x00, 0x01, 0x00];
pub const ETHER_VRRP_V2_DST_MAC: [u8; 6] = [0x01, 0x00, 0x5e, 0x00, 0x00, 0x12];
pub const ETHER_ARP_DST_MAC: [u8; 6] = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
pub const ETHER_FRAME_SIZE: usize = 14;

// ARP Constants
pub const ARP_HW_TYPE: u16 = 1; // ethernet
pub const ARP_OP_REQUEST: u16 = 1; // request

// IP Constants
pub const IP_FRAME_OFFSET: usize = 14;
pub const IP_V4_VERSION: u8 = 0x45;
pub const IP_UPPER_PROTO_VRRP: u8 = 112;
pub const IP_TTL_VRRP_MINTTL: u8 = 255;
pub const IP_DSCP_CS6: u8 = 0xc0;

// VRRP Constants
pub const VRRP_V2_FRAME_OFFSET: usize = 34;
pub const VRRP_V2_VER_TYPE_AUTHMSG: u8 = 0x21;
pub const VRRP_V2_IP_MCAST_DST: [u8; 4] = [224, 0, 0, 18];
pub const VRRP_V2_ADVERT_VERSION_TYPE: u8 = 0x21;
pub const VRRP_V2_DEFAULT_PRIORITY: u8 = 100;
