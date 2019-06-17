//! linux standard c library compatibility

// std, libc, ffi
use std::ffi::{CStr, CString};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

// foreign_types
use foreign_types::{ForeignType, ForeignTypeRef};

// libc-like getifaddrs() function implementation
// Credit sfackler: https://gist.github.com/sfackler/d614e6c130f3462f443e6c0c6255383a
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
