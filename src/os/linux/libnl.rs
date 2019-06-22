//! netlink support module
//! using libnl-3
use crate::*;

// libc
use libc::{c_char, c_int, c_uint, c_void, AF_INET, IF_NAMESIZE};

// std
use std::ffi::CString;
use std::io;

// custom libnl types
// nl_list_head type
#[repr(C)]
#[derive(Debug)]
struct nl_list_head {
    next: *mut c_void,
    prev: *mut c_void,
}

// nl_addr type
#[repr(C)]
#[derive(Debug)]
struct nl_addr {
    a_family: c_int,
    a_maxsize: c_uint,
    a_len: c_uint,
    a_prefixlen: c_int,
    a_refcnt: c_int,
    a_addr: [u8; 4],
}

// rtnl_addr_cacheinfo type
#[repr(C)]
#[derive(Debug)]
struct rtnl_addr_cacheinfo {
    aci_prefered: u32,
    aci_valid: u32,
    aci_cstamp: u32,
    aci_tstamp: u32,
}

// rtnl_addr type
#[repr(C)]
#[derive(Debug)]
struct rtnl_addr {
    // NLHDR_COMMON
    ce_refcnt: c_int,
    ce_ops: *mut c_void,
    ce_cache: *mut c_void,
    ce_list: nl_list_head,
    ce_msgtype: c_int,
    ce_flags: c_int,
    ce_mask: u32,

    a_family: u8,
    a_prefixlen: u8,
    a_flags: u8,
    a_scope: u8,
    a_ifindex: u32,

    a_peer: *mut nl_addr,
    a_local: *mut nl_addr,
    a_bcast: *mut nl_addr,
    a_anycast: *mut nl_addr,
    a_multicast: *mut nl_addr,

    a_cacheinfo: rtnl_addr_cacheinfo,

    a_label: [c_char; IF_NAMESIZE],
    a_flag_mask: u32,
}

// nl_sock enum (hack to represent opaque foreign types)
enum NlSock {}

// Operation public enumerator
pub enum Operation {
    Add, // Add IP address
    Rem, // Remove IP Address
}

// FFI
#[link(name = "nl-3")]
extern "C" {
    // nl_socket_alloc() external function
    // allocate new netlink socket
    fn nl_socket_alloc() -> *mut NlSock;
    // nl_connect() external function
    fn nl_connect(sk: *mut NlSock, protocol: c_int) -> c_int;
    // nl_addr_parse() external function
    fn nl_addr_parse(addrstr: *const c_char, hint: c_int, result: &&mut nl_addr) -> c_int;
}
#[link(name = "nl-route-3")]
extern "C" {
    // rtnl_addr_alloc() external function
    fn rtnl_addr_alloc() -> *mut rtnl_addr;
    // rtnl_addr_set_local() external function
    fn rtnl_addr_set_local(addr: *mut rtnl_addr, local: *mut nl_addr) -> c_int;
    // rtnl_addr_set_ifindex() external function
    fn rtnl_addr_set_ifindex(addr: *mut rtnl_addr, ifindex: i32);
    // rtnl_addr_set_label() external function
    fn rtnl_addr_set_label(addr: *mut rtnl_addr, label: *const c_char);
    // rtnl_addr_add() external function
    // request addition of new address
    fn rtnl_addr_add(sk: *mut NlSock, addr: *mut rtnl_addr, flags: c_int) -> c_int;
    // rtnl_addr_delete() external function
    // request deletion of an address
    fn rtnl_addr_delete(sk: *mut NlSock, addr: *mut rtnl_addr, flags: c_int) -> c_int;
    // rtnl_addr_put() external function
    // free rtnl_addr allocation
    fn rtnl_addr_put(addr: *mut rtnl_addr);
}

// set_ip_address() function
/// Set or remove an IP address on an interface according to the passed Operation variant
pub fn set_ip_address(
    ifindex: i32,
    ifname: &CString,
    ip: [u8; 4],
    netmask: [u8; 4],
    op: Operation,
    debug: &Verbose,
) -> io::Result<()> {
    // null initialize nl_addr 'local'
    let mut local = nl_addr {
        a_family: 0,
        a_maxsize: 0,
        a_len: 0,
        a_prefixlen: 0,
        a_refcnt: 0,
        a_addr: [0; 4],
    };

    // call to external nlsock() function
    let nlsock = unsafe { nl_socket_alloc() };
    if nlsock.is_null() {
        return Err(io::Error::last_os_error());
    }

    // call to external nl_connect() function
    let r = unsafe { nl_connect(nlsock, 0) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }

    // allocate rtnl_addr 'addr'
    let addr = unsafe { rtnl_addr_alloc() };
    if addr.is_null() {
        return Err(io::Error::last_os_error());
    }

    // set ifindex in rtnl_addr 'addr'
    unsafe { rtnl_addr_set_ifindex(addr, ifindex) };

    // set interface label in rtnl_addr 'addr'
    unsafe { rtnl_addr_set_label(addr, ifname.as_ptr()) };

    // convert netmask to prefix length
    // by counting the number of bit set
    // per bytes in netmask array
    let mut prefixlen = 0;
    for b in netmask.iter() {
        prefixlen += b.count_ones();
    }

    // create IP address string
    let ip_str = format!("{}.{}.{}.{}/{}", ip[0], ip[1], ip[2], ip[3], prefixlen);

    // set local IP address in rtnl_addr 'addr'
    let ipaddr = CString::new(ip_str).unwrap();
    let mut local_ptr = &mut local;
    let r = unsafe { nl_addr_parse(ipaddr.as_ptr(), AF_INET, &mut local_ptr) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        DEBUG_SRC_IP,
        format!(
            "ip_slice: {:?}, nl_addr {:?}, result: {}",
            ipaddr, *local_ptr, r
        ),
    );
    let r = unsafe { rtnl_addr_set_local(addr, local_ptr) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }

    // debugging finialized 'addr'
    unsafe {
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_IP,
            format!("addr {:?}", *addr),
        )
    };

    // Perform add or remove operations
    let res: c_int;
    match op {
        Operation::Add => {
            // call external to rtnl_addr_add()
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("calling rtnl_addr_add() with nlsock ptr {:?}", nlsock),
            );
            res = unsafe { rtnl_addr_add(nlsock, addr, 0) };
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("call to rtnl_addr_add() returned {}", res),
            );
        }
        Operation::Rem => {
            // call external to rtnl_addr_delete()
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("calling rtnl_addr_delete() with nlsock ptr {:?}", nlsock),
            );
            res = unsafe { rtnl_addr_delete(nlsock, addr, 0) };
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("call to rtnl_addr_delete() returned {}", res),
            );
        }
    }

    // free allocation of rtnl_addr 'addr'
    unsafe { rtnl_addr_put(addr) };

    // check 'rtnl_addr_add()|del()' returned value
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
