//! Constants module
//! This module regroups all the program's and protocols constants.

// Program Constants
pub const RVRRPD_BANNER: &str = r"
   __      _______  _____  _____    _ 
   \ \    / /  __ \|  __ \|  __ \  | |
 _ _\ \  / /| |__) | |__) | |__) |_| |
| '__\ \/ / |  _  /|  _  /|  ___/ _` |
| |   \  /  | | \ \| | \ \| |  | (_| |
|_|    \/   |_|  \_\_|  \_\_|   \__,_|
";
pub const RVRRPD_COPYRIGHT_BLOCK: &str = r"
Copyright (C) 2019-2023 Nicolas Chabbey.
License GPLv3+: GNU GPL Version 3 or any later version <https://www.gnu.org/licenses/gpl-3.0.txt>.
This program comes with ABSOLUTELY NO WARRANTY. This is free software,
and you are welcome to redistribute it under certain conditions.";
pub const RVRRPD_DFLT_CFG_FILE: &str = "/etc/rvrrpd/rvrrpd.conf";
pub const RVRRPD_DFLT_PIDFILE: &str = "/var/run/rvrrpd.pid";
pub const RVRRPD_DFLT_WORKDIR: &str = "/tmp";
pub const RVRRPD_DFLT_LOGFILE: &str = "/var/log/rvrrpd.log";
pub const RVRRPD_DFLT_ELOGFILE: &str = "/var/log/rvrrpd-error.log";
pub const RVRRPD_DFLT_DATE_FORMAT: &str = "%b %e %Y %T";
pub const RVRRPD_DFLT_MACVLAN_NAME: &str = "standby";
pub const RVRRPD_DFLT_CLIENT_API: &str = "disabled";
pub const RVRRPD_NAME: &str = env!("CARGO_PKG_NAME");
pub const RVRRPD_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RVRRPD_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const RVRRPD_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
pub const RVRRPD_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

// Config Constants
pub const RVRRPD_CFG_DFLT_TLSKEY: &str = "/etc/rvrrpd/ssl/key.pem";
pub const RVRRPD_CFG_DFLT_TLSCERT: &str = "/etc/rvrrpd/ssl/cert.pem";

// Debug Constants
pub const DEBUG_LEVEL_INFO: u8 = 0;
pub const DEBUG_LEVEL_LOW: u8 = 1;
pub const DEBUG_LEVEL_MEDIUM: u8 = 2;
pub const DEBUG_LEVEL_HIGH: u8 = 3;
pub const DEBUG_LEVEL_EXTENSIVE: u8 = 5;
pub const DEBUG_SRC_INFO: &str = "info";
pub const DEBUG_SRC_PROTO: &str = "protocols";
pub const DEBUG_SRC_VR: &str = "vr";
pub const DEBUG_SRC_MAIN: &str = "main";
pub const DEBUG_SRC_MAC: &str = "mac";
pub const DEBUG_SRC_ROUTE: &str = "route";
pub const DEBUG_SRC_PACKET: &str = "packet";
pub const DEBUG_SRC_ARP: &str = "arp";
pub const DEBUG_SRC_THREAD: &str = "thread";
pub const DEBUG_SRC_THREADP: &str = "thread-pool";
pub const DEBUG_SRC_FSM: &str = "fsm";
pub const DEBUG_SRC_WORKER: &str = "worker";
pub const DEBUG_SRC_WORKERG: &str = "worker-reg";
pub const DEBUG_SRC_TIMER: &str = "timer";
pub const DEBUG_SRC_IP: &str = "ip";
pub const DEBUG_SRC_AUTH: &str = "auth";
pub const DEBUG_SRC_MACVLAN: &str = "macvlan";
pub const DEBUG_SRC_BPF: &str = "bpf";

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
pub const VRRP_V2_CHECKSUM_POS: usize = 6;
pub const VRRP_V2_VER_TYPE_AUTHMSG: u8 = 0x21;
pub const VRRP_V2_IP_MCAST_DST: [u8; 4] = [224, 0, 0, 18];
pub const VRRP_V2_ADVERT_VERSION_TYPE: u8 = 0x21;
pub const VRRP_V2_DEFAULT_PRIORITY: u8 = 100;
pub const VRRP_V2_TIMER_MCANCELLED: f32 = 65535.0;
pub const VRRP_V2_TIMER_ACANCELLED: u8 = 255;

// Authentication Constants
pub const AUTH_TYPE_SIMPLE: u8 = 1;
pub const AUTH_TYPE_P0: u8 = 250;
pub const AUTH_TYPE_P1: u8 = 251;
