//! configuration file handling module
//! This module provides structure and methods related to configuration file handling.
use super::*;

// std
use std::net::IpAddr;

/// CfgType Enumerator
pub enum CfgType {
    Toml, // TOML
    Json, // JSON
}

/// Main Configuration Structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CConfig {
    pub debug: Option<u8>,
    pub time_zone: Option<String>,
    pub time_format: Option<String>,
    pub pid: Option<String>,
    pub working_dir: Option<String>,
    pub main_log: Option<String>,
    pub error_log: Option<String>,
    pub vrouter: Option<Vec<VRConfig>>,
    pub protocols: Option<Protocols>,
    pub client_api: Option<String>,
    pub api: Option<API>,
}

impl CConfig {
    // debug() getter
    pub fn debug(&self) -> u8 {
        match self.debug {
            Some(v) => v,
            None => DEBUG_LEVEL_INFO,
        }
    }
    // time_zone() getter
    pub fn time_zone(&self) -> u8 {
        match &self.time_zone {
            Some(s) => match &s[..] {
                "local" => 0,
                "utc" => 1,
                _ => 0,
            },
            None => 0,
        }
    }
    // time_format() getter
    pub fn time_format(&self) -> u8 {
        match &self.time_format {
            Some(s) => match &s[..] {
                "disabled" => 0,
                "short" => 1,
                "rfc2822" => 2,
                _ => 0,
            },
            None => 0,
        }
    }
    // pid() getter
    pub fn pid(&self) -> String {
        match &self.pid {
            Some(v) => v.clone(),
            None => RVRRPD_DFLT_PIDFILE.to_string(),
        }
    }
    // working_dir() getter
    pub fn working_dir(&self) -> String {
        match &self.working_dir {
            Some(v) => v.clone(),
            None => RVRRPD_DFLT_WORKDIR.to_string(),
        }
    }
    // main_log() getter
    pub fn main_log(&self) -> String {
        match &self.main_log {
            Some(v) => v.clone(),
            None => RVRRPD_DFLT_LOGFILE.to_string(),
        }
    }
    // error_log() getter
    pub fn error_log(&self) -> String {
        match &self.error_log {
            Some(v) => v.clone(),
            None => RVRRPD_DFLT_ELOGFILE.to_string(),
        }
    }
    // client_api() method
    pub fn client_api(&self) -> bool {
        match &self.client_api {
            Some(s) => match &s[..] {
                "http" => true,
                _ => true,
            },
            None => false,
        }
    }
}

/// Virtual-Routers Configuration Structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VRConfig {
    group: u8,
    interface: String,
    vip: Option<String>,
    priority: Option<u8>,
    preemption: Option<bool>,
    auth_type: Option<String>,
    auth_secret: Option<String>,
    timers: Option<Timers>,
    rfc3768: Option<bool>,
    netdrv: Option<String>,
    iftype: Option<String>,
    vifname: Option<String>,
    socket_filter: Option<bool>,
}
impl VRConfig {
    // group() getter
    pub fn group(&self) -> u8 {
        if self.group < 1 {
            panic!("error(config): Please configure a group id between 1 and 255")
        }
        self.group
    }
    // interface() getter
    pub fn interface(&self) -> &String {
        &self.interface
    }
    // vip() getter
    pub fn vip(&self) -> [u8; 4] {
        match &self.vip {
            Some(ip) => match ip.parse::<IpAddr>().unwrap() {
                IpAddr::V4(ip) => ip.octets(),
                IpAddr::V6(_ipv6) => panic!("error(config): Only IPv4 addresses are supported"),
            },
            None => panic!("error(config): No virtual IP specified"),
        }
    }
    // timer_advert() getter
    pub fn timer_advert(&self) -> u8 {
        match &self.timers {
            Some(t) => t.advert,
            None => 1,
        }
    }
    // priority() getter
    pub fn priority(&self) -> u8 {
        match self.priority {
            Some(v) => {
                if v < 1 || v > 254 {
                    panic!("error(config): Please configure a priority between 1 and 254");
                }
                v
            }
            None => VRRP_V2_DEFAULT_PRIORITY,
        }
    }
    // preemption() getter
    pub fn preemption(&self) -> bool {
        match self.preemption {
            Some(b) => b,
            None => false,
        }
    }
    // auth_type() method
    pub fn auth_type(&self) -> u8 {
        match &self.auth_type {
            Some(s) => match &s[..] {
                "rfc2338-simple" => AUTH_TYPE_SIMPLE,
                "p0-t8-sha256" => AUTH_TYPE_P0,
                "p1-b8-shake256" => AUTH_TYPE_P1,
                _ => panic!("error(config): authentication type {} is not supported", s),
            },
            None => 0,
        }
    }
    // auth_secret() method
    pub fn auth_secret(&self) -> Option<String> {
        match &self.auth_secret {
            Some(cs) => match self.auth_type() {
                // if type-1, then truncate to 8 bytes
                1 => {
                    let mut s = cs.clone();
                    s.truncate(8);
                    Option::Some(s)
                }
                _ => {
                    let s = cs.clone();
                    Option::Some(s)
                }
            },
            None => Option::None,
        }
    }
    // rfc3768() getter
    pub fn rfc3768(&self) -> bool {
        // if auth_type is 'p0-t8-sha256', or 'p1-b8-shake256',
        // overwrite rfc3768 compatibility flag
        match &self.auth_type {
            Some(t) => match &t[..] {
                "p0-t8-sha256" | "p1-b8-shake256" => {
                    println!(
                        "warning(config): authentication type {} is enabled, forcing rfc3768 compatibility.",
                        t
                    );
                    return true;
                }
                _ => {}
            },
            None => {}
        }
        match self.rfc3768 {
            Some(b) => b,
            None => true,
        }
    }
    // netdrv() method
    pub fn netdrv(&self) -> NetDrivers {
        // if os is Linux
        if cfg!(target_os = "linux") {
            // if macvlan is configured, return libnl
            match self.iftype() {
                IfTypes::macvlan => return NetDrivers::libnl,
                _ => {}
            }
            match &self.netdrv {
                Some(s) => match &s[..] {
                    "ioctl" => NetDrivers::ioctl,
                    _ => NetDrivers::libnl,
                },
                None => NetDrivers::libnl,
            }
        }
        // unspecified OSes
        else {
            NetDrivers::ioctl
        }
    }
    // iftype() method
    pub fn iftype(&self) -> IfTypes {
        // if os is Linux
        if cfg!(target_os = "linux") {
            match &self.iftype {
                Some(s) => match &s[..] {
                    "macvlan" => IfTypes::macvlan,
                    _ => IfTypes::ether,
                },
                None => IfTypes::ether,
            }
        } else {
            IfTypes::ether
        }
    }
    // vifname() method
    pub fn vifname(&self) -> String {
        match &self.vifname {
            Some(s) => s.clone(),
            None => format!("{}{}", RVRRPD_DFLT_MACVLAN_NAME, self.group),
        }
    }
    // socket_filter() method
    pub fn socket_filter(&self) -> bool {
        match self.socket_filter {
            Some(b) => b,
            None => true,
        }
    }
}

/// Timers Option Type
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Timers {
    advert: u8,
}
impl Default for Timers {
    fn default() -> Self {
        Timers { advert: 1 }
    }
}

/// Protocols Option Type
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Protocols {
    pub r#static: Option<Vec<Static>>,
}

/// Static Option Type
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Static {
    route: String,
    mask: String,
    nh: String,
    metric: Option<u16>,
    mtu: Option<u16>,
}

// Static Option Implementation
impl Static {
    // route() getter
    // convert IPv4 String to array of four 8-bits unsigned integers
    pub fn route(&self) -> [u8; 4] {
        match self.route.parse::<IpAddr>().unwrap() {
            IpAddr::V4(ip) => ip.octets(),
            IpAddr::V6(_ipv6) => panic!("error(config-static): Only IPv4 routes are supported"),
        }
    }
    // mask() getter
    pub fn mask(&self) -> [u8; 4] {
        match self.mask.parse::<IpAddr>().unwrap() {
            IpAddr::V4(ip) => ip.octets(),
            IpAddr::V6(_ipv6) => panic!("error(config-static): Only IPv4 masks are supported"),
        }
    }
    // nh() getter
    pub fn nh(&self) -> [u8; 4] {
        match self.nh.parse::<IpAddr>().unwrap() {
            IpAddr::V4(ip) => ip.octets(),
            IpAddr::V6(_ipv6) => panic!("error(config-static): Only IPv4 next-hops are supported"),
        }
    }
    // metric() getter
    pub fn metric(&self) -> i16 {
        match self.metric {
            Some(v) => v as i16,
            None => 0,
        }
    }
    // mtu() getter
    pub fn mtu(&self) -> u64 {
        match self.mtu {
            Some(v) => v as u64,
            None => 0,
        }
    }
}

// decode_config() function
/// read and decode configuration file
pub fn decode_config(filename: String, cfgtype: CfgType) -> CConfig {
    let file: std::string::String = match std::fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "error(config): Cannot read rVRRPd configuration file: {}",
                e
            );
            std::process::exit(1);
        }
    };
    match cfgtype {
        // TOML
        CfgType::Toml => {
            let config: CConfig = match toml::from_str(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error(config): Cannot parse TOML configuration file: {}", e);
                    std::process::exit(1);
                }
            };
            return config;
        }
        // JSON
        CfgType::Json => {
            let config: CConfig = match serde_json::from_str(&file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("error(config): Cannot parse JSON configuration file: {}", e);
                    std::process::exit(1);
                }
            };
            return config;
        }
    }
}

/// API structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct API {
    users: Vec<String>,
}

// API structure implementation
impl API {
    // users() method
    pub fn users(&self) -> Vec<String> {
        self.users.clone()
    }
}
