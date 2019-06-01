//! linux specific network device functions module
//! This module interfaces with the linux netdevice kernel API and related networking functions of the standard C library.
use super::*;

// std, libc, ffi
use libc::{
    c_short, c_uchar, c_ulong, c_ushort, ioctl, AF_INET, ARPHRD_ETHER, ETH_ALEN, IF_NAMESIZE,
    RTF_UP,
};
use std::ffi::{CStr, CString};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

// foreign_types
use foreign_types::{ForeignType, ForeignTypeRef};

// flags values (/usr/include/net/if.h)
const IFF_UP: c_short = 0x01;
const IFF_RUNNING: c_short = 0x40;
const IFF_PROMISC: c_short = 0x100;

/// ioctl_flags Structure
#[repr(C)]
struct ioctl_flags {
    ifr_name: [u8; IF_NAMESIZE],
    ifr_flags: c_short,
}

/// ioctl_4_addr Structure
#[repr(C)]
struct ioctl_v4_addr {
    ifr_name: [u8; IF_NAMESIZE],
    ifr_addr: int_sockaddr_pad,
}

/// ioctl_v4_netmask Structure
#[repr(C)]
struct ioctl_v4_netmask {
    ifr_name: [u8; IF_NAMESIZE],
    ifr_netmask: int_sockaddr_pad,
}

/// ioctl_v4_route Structure
#[derive(Debug)]
#[repr(C)]
struct ioctl_v4_route {
    rt_hash: c_ulong,
    rt_dst: int_sockaddr,
    rt_gateway: int_sockaddr,
    rt_genmask: int_sockaddr,
    rt_flags: c_ushort,
    rt_pad1: c_short,
    rt_pad2: c_ulong,
    rt_tos: c_uchar,
    rt_class: c_uchar,
    rt_pad3: [c_short; 3], // c_short or x3 on 64-bits
    rt_metric: c_short,
    rt_dev: *const u8,
    rt_mtu: c_ulong,
    rt_window: c_ulong,
    rt_irtt: c_ushort,
}

/// ioctl_ether_mac Structure
#[repr(C)]
#[derive(Debug)]
struct ioctl_ether_mac {
    ifr_name: [u8; IF_NAMESIZE],
    ifr_hwaddr: int_sockaddr_ether,
}

/// internal int_sockaddr_pad Structure
#[repr(C)]
#[derive(Debug)]
struct int_sockaddr_pad {
    sa_family: u16,
    sa_void: u16, // ensure proper alignement
    sa_data: [u8; 14],
}

/// internal int_sockaddr Structure
#[derive(Debug)]
#[repr(C)]
struct int_sockaddr {
    sa_family: u16,
    sa_data: [u8; 14],
}

/// internal int_sockaddr_ether Structure
#[derive(Debug)]
#[repr(C)]
struct int_sockaddr_ether {
    sa_family: u16,
    sa_data: [u8; ETH_ALEN as usize],
}

//// pflag operation Enumerator
pub enum PflagOp {
    Set,
    Unset,
}

// set_if_promiscuous() function
/// Set (or Unset) interface in promiscuous mode
pub fn set_if_promiscuous(sockfd: i32, ifname: &CString, op: PflagOp) -> io::Result<()> {
    // create a slice of mutable reference to array of 16 u8
    let ifname_slice = &mut [0u8; 16];

    // for every bytes/character in name of type Cstring, insert it into the above slice.
    for (i, b) in ifname.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b;
    }

    // check interface name size
    if ifname_slice.len() > IF_NAMESIZE {
        panic!("Interface name is longer than {}", IF_NAMESIZE - 1);
    }

    // construct ioctl_flags structure
    let mut ifopts = ioctl_flags {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            // the src and dst must be of the same size.
            buf.clone_from_slice(ifname_slice);
            buf
        },
        ifr_flags: 0,
    };

    // operation to perform on promiscuous flag
    match op {
        PflagOp::Set => {
            // set the flags to UP,RUNNING,PROMISC using bitwise OR operation.
            ifopts.ifr_flags |= IFF_UP | IFF_RUNNING | IFF_PROMISC;
            let res = unsafe { ioctl(sockfd, libc::SIOCSIFFLAGS, &mut ifopts) };
            if res < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        PflagOp::Unset => {
            // unset PROMISC flag
            ifopts.ifr_flags |= IFF_UP | IFF_RUNNING;
            let res = unsafe { ioctl(sockfd, libc::SIOCSIFFLAGS, &mut ifopts) };
            if res < 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }

    Ok(())
}

// set_ip_address() function
/// Set an IP address on an interface
pub fn set_ip_address(
    sockfd: i32,
    ifname: &CString,
    ip: [u8; 4],
    netmask: [u8; 4],
) -> io::Result<()> {
    // create a slice of mutable reference to array of 16 u8
    let ifname_slice = &mut [0u8; 16];
    // for every bytes/character in name of type Cstring, insert it into the above slice.
    for (i, b) in ifname.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b;
    }
    // check interface name size
    if ifname_slice.len() > IF_NAMESIZE {
        panic!("Interface name is longer than {}", IF_NAMESIZE - 1);
    }

    // create IP address slice
    let ip_addr_slice = &mut [0u8; 14];
    for (i, b) in ip.iter().enumerate() {
        ip_addr_slice[i] = *b;
    }

    // create IP netmask slice
    let ip_netmask_slice = &mut [0u8; 14];
    for (i, b) in netmask.iter().enumerate() {
        ip_netmask_slice[i] = *b;
    }

    // construct ifaddr structure
    let mut ifaddr = ioctl_v4_addr {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            // the src and dst must be of the same size.
            buf.clone_from_slice(ifname_slice);
            buf
        },
        ifr_addr: {
            let mut ip_buf = [0u8; 14];
            ip_buf.clone_from_slice(ip_addr_slice);
            let addr = int_sockaddr_pad {
                sa_family: AF_INET as u16,
                sa_void: 0,
                sa_data: ip_buf,
            };
            addr
        },
    };

    // construct ifnetmask structure
    let mut ifnetmask = ioctl_v4_netmask {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            // the src and dst must be of the same size.
            buf.clone_from_slice(ifname_slice);
            buf
        },
        ifr_netmask: {
            let mut netmask_buf = [0u8; 14];
            netmask_buf.clone_from_slice(ip_netmask_slice);
            let netmask = int_sockaddr_pad {
                sa_family: AF_INET as u16,
                sa_void: 0,
                sa_data: netmask_buf,
            };
            netmask
        },
    };

    // ioctl - set interface's IP address
    let res = unsafe { ioctl(sockfd, libc::SIOCSIFADDR, &mut ifaddr) };
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    // ioctl - set interface's netmask
    let res = unsafe { ioctl(sockfd, libc::SIOCSIFNETMASK, &mut ifnetmask) };
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

// get_mac_addr() function
/// Get the MAC address of an interface
/// this function return the interface's MAC address if read sucessfully
pub fn get_mac_addr(sockfd: i32, ifname: &CString, debug_level: u8) -> io::Result<[u8; 6]> {
    // convert interface name to CString type
    let ifname = CString::new(ifname.as_bytes() as &[u8]).unwrap();

    // create a slice of mutable reference to array of 16 u8
    let ifname_slice = &mut [0u8; 16];

    // for every bytes/character in name of type Cstring, insert it into the above slice.
    for (i, b) in ifname.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b;
    }

    // check interface name size
    if ifname_slice.len() > IF_NAMESIZE {
        panic!("Interface name is longer than {}", IF_NAMESIZE - 1);
    }

    // constuct ifmac structure
    let mut ifmac = ioctl_ether_mac {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            // the src and dst must be of the same size.
            buf.clone_from_slice(ifname_slice);
            buf
        },
        ifr_hwaddr: {
            let mac_buf = [0u8; ETH_ALEN as usize];
            let mac = int_sockaddr_ether {
                sa_family: 0,
                sa_data: mac_buf,
            };
            mac
        },
    };

    // ioctl - set/reset MAC address
    print_debug(
        debug_level,
        DEBUG_LEVEL_HIGH,
        format!("debug(mac): getting mac address on interface {:?}", ifname),
    );
    let result = unsafe { ioctl(sockfd, libc::SIOCGIFHWADDR, &mut ifmac) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    print_debug(
        debug_level,
        DEBUG_LEVEL_HIGH,
        format!(
            "debug(mac): got interface {:?} mac address: {:?}",
            ifname, ifmac
        ),
    );

    // return the mac address
    Ok(ifmac.ifr_hwaddr.sa_data)
}

// set_mac_addr() function
/// Set the specified MAC address on interface
pub fn set_mac_addr(
    sockfd: i32,
    ifname: &CString,
    mac: [u8; 6],
    debug_level: u8,
) -> io::Result<()> {
    // convert interface name to CString type
    let ifname = CString::new(ifname.as_bytes() as &[u8]).unwrap();

    // create a slice of mutable reference to array of 16 u8
    let ifname_slice = &mut [0u8; 16];

    // for every bytes/character in name of type Cstring, insert it into the above slice.
    for (i, b) in ifname.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b;
    }

    // check interface name size
    if ifname_slice.len() > IF_NAMESIZE {
        panic!("Interface name is longer than {}", IF_NAMESIZE - 1);
    }

    // create ethernet mac slice
    let ether_mac_slice = &mut [0u8; ETH_ALEN as usize];
    for (i, b) in mac.iter().enumerate() {
        ether_mac_slice[i] = *b;
    }

    // constuct ifmac structure
    let mut ifmac = ioctl_ether_mac {
        ifr_name: {
            let mut buf = [0u8; IF_NAMESIZE];
            // the src and dst must be of the same size.
            buf.clone_from_slice(ifname_slice);
            buf
        },
        ifr_hwaddr: {
            let mut mac_buf = [0u8; ETH_ALEN as usize];
            mac_buf.clone_from_slice(ether_mac_slice);
            let mac = int_sockaddr_ether {
                sa_family: ARPHRD_ETHER as u16,
                sa_data: mac_buf,
            };
            mac
        },
    };

    // ioctl - set/reset MAC address
    print_debug(
        debug_level,
        DEBUG_LEVEL_HIGH,
        format!("debug(mac): setting mac address {:?}: {:?}", ifname, ifmac),
    );
    let result = unsafe { ioctl(sockfd, libc::SIOCSIFHWADDR, &mut ifmac) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

// set_ip_routes() function
/// Set IP routes into the routing table
/// if boolean 'set_flag' is true then add the route, otherwise remove
pub fn set_ip_route(
    sockfd: i32,
    ifname: &String,
    route: [u8; 4],
    rtmask: [u8; 4],
    gw: [u8; 4],
    metric: i16,
    mtu: u64,
    set_flag: bool,
    debug_level: u8,
) -> io::Result<()> {
    // convert interface name to CString type
    let ifname = CString::new(ifname.as_bytes() as &[u8]).unwrap();

    // create a slice of mutable reference to array of 16 u8
    let ifname_slice = &mut [0u8; 16];

    // for every bytes/character in name of type Cstring, insert it into the above slice.
    for (i, b) in ifname.as_bytes_with_nul().iter().enumerate() {
        ifname_slice[i] = *b;
    }

    // check interface name size
    if ifname_slice.len() > IF_NAMESIZE {
        panic!("Interface name is longer than {}", IF_NAMESIZE - 1);
    }

    // create route slice
    let route_slice = &mut [0u8; 14];
    for (i, b) in route.iter().enumerate() {
        route_slice[i + 2] = *b;
    }

    // create rtmask slice
    let rtmask_slice = &mut [0u8; 14];
    for (i, b) in rtmask.iter().enumerate() {
        rtmask_slice[i + 2] = *b;
    }

    // create gateway slice
    let gateway_slice = &mut [0u8; 14];
    for (i, b) in gw.iter().enumerate() {
        gateway_slice[i + 2] = *b;
    }

    // construct route
    let dst_route = int_sockaddr {
        sa_family: AF_INET as u16,
        //sa_void: 0,
        sa_data: {
            let mut route_buf = [0u8; 14];
            route_buf.clone_from_slice(route_slice);
            route_buf
        },
    };

    // construct route mask
    let route_mask = int_sockaddr {
        sa_family: AF_INET as u16,
        //sa_void: 0,
        sa_data: {
            let mut mask_buf = [0u8; 14];
            mask_buf.clone_from_slice(rtmask_slice);
            mask_buf
        },
    };

    // construct gateway
    let gateway = int_sockaddr {
        sa_family: AF_INET as u16,
        //sa_void: 0,
        sa_data: {
            let mut gateway_buf = [0u8; 14];
            gateway_buf.clone_from_slice(gateway_slice);
            gateway_buf
        },
    };

    // set device name
    let mut dev = [0u8; IF_NAMESIZE];
    dev.clone_from_slice(ifname_slice);

    // construct ifroute structure
    let mut ifroute = ioctl_v4_route {
        rt_hash: 0,
        rt_dst: dst_route,      // set route
        rt_gateway: gateway,    // set gateway
        rt_genmask: route_mask, // set route's mask
        rt_flags: 0,            // initialize flags to zero
        rt_pad1: 0,
        rt_pad2: 0,
        rt_tos: 0,
        rt_class: 0,
        rt_pad3: [0, 0, 0],
        rt_metric: metric,         // set metric
        rt_dev: &dev as *const u8, // set dev
        rt_mtu: mtu,
        rt_window: 0,
        rt_irtt: 0,
    };
    // set route flags
    ifroute.rt_flags |= RTF_UP | libc::RTF_GATEWAY;

    // ioctl - set/delete route
    let result: i32;
    if set_flag == true {
        print_debug(
            debug_level,
            DEBUG_LEVEL_HIGH,
            format!("debug(route): adding route {:?}", ifroute),
        );
        result = unsafe { ioctl(sockfd, libc::SIOCADDRT, &mut ifroute) };
    } else {
        print_debug(
            debug_level,
            DEBUG_LEVEL_HIGH,
            format!("debug(route): removing route {:?}", ifroute),
        );
        result = unsafe { ioctl(sockfd, libc::SIOCDELRT, &mut ifroute) };
    }
    if result < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

// libc-like getifaddrs() function implementation
/// Credit to sfackler: https://gist.github.com/sfackler/d614e6c130f3462f443e6c0c6255383a
foreign_type! {
    #[derive(Debug)]
    pub type IfAddrs: Sync + Send {
        type CType = libc::ifaddrs;
        fn drop = libc::freeifaddrs;
    }
}

impl IfAddrs {
    pub fn get() -> io::Result<IfAddrs> {
        unsafe {
            let mut ifaddrs = ptr::null_mut();
            let r = libc::getifaddrs(&mut ifaddrs);
            if r == 0 {
                Ok(IfAddrs::from_ptr(ifaddrs))
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl IfAddrsRef {
    // next() method
    pub fn next(&self) -> Option<&IfAddrsRef> {
        unsafe {
            let next = (*self.as_ptr()).ifa_next;
            if next.is_null() {
                None
            } else {
                Some(IfAddrsRef::from_ptr(next))
            }
        }
    }

    // name() method
    pub fn name(&self) -> &str {
        unsafe {
            let s = CStr::from_ptr((*self.as_ptr()).ifa_name);
            s.to_str().unwrap()
        }
    }

    // addr() method
    pub fn addr(&self) -> Option<IpAddr> {
        unsafe {
            let addr = (*self.as_ptr()).ifa_addr;
            if addr.is_null() {
                return None;
            }

            match (*addr).sa_family as _ {
                libc::AF_INET => {
                    let addr = addr as *mut libc::sockaddr_in;
                    let addr = Ipv4Addr::from((*addr).sin_addr.s_addr.to_be());
                    Some(IpAddr::V4(addr))
                }
                libc::AF_INET6 => {
                    let addr = addr as *mut libc::sockaddr_in6;
                    let addr = Ipv6Addr::from((*addr).sin6_addr.s6_addr);
                    Some(IpAddr::V6(addr))
                }
                _ => None,
            }
        }
    }

    // netmask() method
    pub fn netmask(&self) -> Option<IpAddr> {
        unsafe {
            let netmask = (*self.as_ptr()).ifa_netmask;
            if netmask.is_null() {
                return None;
            }

            match (*netmask).sa_family as _ {
                libc::AF_INET => {
                    let netmask = netmask as *mut libc::sockaddr_in;
                    let netmask = Ipv4Addr::from((*netmask).sin_addr.s_addr.to_be());
                    Some(IpAddr::V4(netmask))
                }
                libc::AF_INET6 => {
                    let netmask = netmask as *mut libc::sockaddr_in6;
                    let netmask = Ipv6Addr::from((*netmask).sin6_addr.s6_addr);
                    Some(IpAddr::V6(netmask))
                }
                _ => None,
            }
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter(Some(self))
    }
}

impl<'a> IntoIterator for &'a IfAddrs {
    type Item = &'a IfAddrsRef;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a IfAddrsRef {
    type Item = &'a IfAddrsRef;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

pub struct Iter<'a>(Option<&'a IfAddrsRef>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a IfAddrsRef;

    fn next(&mut self) -> Option<&'a IfAddrsRef> {
        let cur = match self.0 {
            Some(cur) => cur,
            None => return None,
        };

        self.0 = cur.next();
        Some(cur)
    }
}

// c_ifnametoindex() function
/// see 'man 3 if_nametoindex'
pub fn c_ifnametoindex(ifname: &String) -> io::Result<u32> {
    unsafe {
        let c_ifname = CString::new(ifname.clone()).unwrap();
        let r = libc::if_nametoindex(c_ifname.as_ptr());
        if r == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(r)
        }
    }
}

// Tests
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_ifaddrs_type() {
        let addrs = IfAddrs::get().unwrap();
        println!(
            "{:?}",
            addrs
                .iter()
                .map(|a| (a.name(), a.addr()))
                .collect::<Vec<_>>()
        );
    }
}
