//! netlink support module
//! using libnl-3
use crate::*;

// libc
use libc::{c_char, c_int, c_uint, c_void, AF_INET, AF_LLC, ETH_ALEN, IF_NAMESIZE};

// std
use std::ffi::CString;
use std::io;

// constants
const INT_RTAX_MAX: usize = 8; // to verify __RTAX_MAX enum value
const INT_NLM_F_CREATE: i32 = 0x400; // include/linux/netlink.h

// operating system drivers
use crate::os::drivers::Operation;

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

// rtnl_rtcacheinfo type
#[repr(C)]
#[derive(Debug)]
struct rtnl_rtcacheinfo {
    rtci_clntref: u32,
    rtci_last_use: u32,
    rtci_expires: u32,
    rtci_error: i32,
    rtci_used: u32,
    rtci_id: u32,
    rtci_ts: u32,
    rtci_tsafe: u32,
}

// rtnl_route type
#[repr(C)]
#[derive(Debug)]
struct rtnl_route {
    // NLHDR_COMMON
    ce_refcnt: c_int,
    ce_ops: *mut c_void,
    ce_cache: *mut c_void,
    ce_list: nl_list_head,
    ce_msgtype: c_int,
    ce_flags: c_int,
    ce_mask: u32,

    rt_family: u8,
    rt_dst_len: u8,
    rt_src_len: u8,
    rt_tos: u8,
    rt_table: u8,
    rt_protocol: u8,
    rt_scope: u8,
    rt_type: u8,
    rt_flags: u32,

    rt_dst: *mut nl_addr,
    rt_src: *mut nl_addr,

    rt_iiff: [c_char; IF_NAMESIZE],
    rt_oif: u32,
    rt_gateway: *mut nl_addr,
    rt_prio: u32,
    rt_metrics: [u32; INT_RTAX_MAX],
    rt_metrics_mask: u32,

    rt_pref_src: *mut nl_addr,
    rt_nexthops: *mut nl_list_head,
    rt_realms: u32, // substitue for realm_t as returned by rtnl_rule_get_realms(),
    rt_cacheinfo: rtnl_rtcacheinfo,
    rt_mp_algo: u32,
    rt_flag_mask: u32,
}

// rtnl_nexhop type
#[repr(C)]
#[derive(Debug)]
struct rtnl_nexthop {
    rtnh_flags: u8,
    rtnh_flag_mask: u8,
    rtnh_weight: u8,
    rtnh_ifindex: u32,
    rtnh_gateway: *mut nl_addr,
    ce_mask: u32,
    rtnh_list: nl_list_head,
    rtnh_realms: u32,
}

// int_ether_addr type
#[repr(C)]
#[derive(Debug)]
struct int_ether_addr {
    ether_addr: [u8; ETH_ALEN as usize],
}

// nl_sock enum (hack to represent opaque foreign types)
enum NlSock {}

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
    // nl_addr_put() external function
    fn nl_addr_put(addr: *mut nl_addr);
    // nl_addr_build() external function
    fn nl_addr_build(family: c_int, buf: *const int_ether_addr, size: i32) -> *mut nl_addr;
}
#[link(name = "nl-route-3")]
extern "C" {
    // ** libnl - addresses
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

    // ** libnl - routing
    // rtnl_route_alloc() external function
    fn rtnl_route_alloc() -> *mut rtnl_route;
    // rtnl_route_add() external function
    fn rtnl_route_add(sk: *mut NlSock, route: *mut rtnl_route, flags: c_int) -> c_int;
    // rtnl_route_delete() external function
    fn rtnl_route_delete(sk: *mut NlSock, route: *mut rtnl_route, flags: c_int) -> c_int;
    // // rtnl_route_set_iif() external function
    // fn rtnl_route_set_iif(route: *mut rtnl_route, ifindex: i32);
    // rtnl_route_set_dst() external function
    fn rtnl_route_set_dst(route: *mut rtnl_route, addr: *mut nl_addr) -> c_int;
    // rtnl_route_add_nexthop() external function
    fn rtnl_route_add_nexthop(route: *mut rtnl_route, nh: *mut rtnl_nexthop);
    // rtnl_route_set_metric() external function
    fn rtnl_route_set_metric(route: *mut rtnl_route, metric: i32, value: u32) -> c_int;
    // rtnl_route_nh_alloc() external function
    fn rtnl_route_nh_alloc() -> *mut rtnl_nexthop;
    // rtnl_route_nh_set_gateway() external function
    fn rtnl_route_nh_set_gateway(nh: *mut rtnl_nexthop, addr: *mut nl_addr);
    // // rtnl_route_nh_set_ifindex() external function
    // fn rtnl_route_nh_set_ifindex(nh: *mut rtnl_nexthop, int: i32);
    // rtnl_route_put() external function
    // free rtnl_route allocation
    fn rtnl_route_put(route: *mut rtnl_route);

    // ** libnl - link
    // rtnl_link_macvlan_alloc() external function
    fn rtnl_link_macvlan_alloc() -> *mut c_void;
    // rtnl_link_set_link() external function
    fn rtnl_link_set_link(link: *mut c_void, ifindex: i32);
    // rtnl_link_set_addr() external function
    fn rtnl_link_set_addr(link: *mut c_void, addr: *mut nl_addr);
    // rtnl_link_macvlan_set_mode() external function
    fn rtnl_link_macvlan_set_mode(link: *mut c_void, mode: u32) -> c_int;
    // rtnl_link_macvlan_str2mode() external function
    fn rtnl_link_macvlan_str2mode(name: &[u8]) -> c_uint;
    // rtnl_link_add() external function
    fn rtnl_link_add(sk: *mut NlSock, link: *mut c_void, flags: c_int) -> c_int;
    // rtnl_link_put() external function
    fn rtnl_link_put(link: *mut c_void);
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

    // allocate rtnl_addr 'nladdr'
    let nladdr = unsafe { rtnl_addr_alloc() };
    if nladdr.is_null() {
        return Err(io::Error::last_os_error());
    }

    // set ifindex in rtnl_addr 'nladdr'
    unsafe { rtnl_addr_set_ifindex(nladdr, ifindex) };

    // set interface label in rtnl_addr 'nladdr'
    unsafe { rtnl_addr_set_label(nladdr, ifname.as_ptr()) };

    // null initialize nl_addr 'local'
    let mut laddr = nl_addr {
        a_family: 0,
        a_maxsize: 0,
        a_len: 0,
        a_prefixlen: 0,
        a_refcnt: 0,
        a_addr: [0; 4],
    };

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
    let mut laddr_ptr = &mut laddr;
    let r = unsafe { nl_addr_parse(ipaddr.as_ptr(), AF_INET, &mut laddr_ptr) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        DEBUG_SRC_IP,
        format!(
            "ip_slice: {:?}, nl_addr {:?}, result: {}",
            ipaddr, *laddr_ptr, r
        ),
    );
    let r = unsafe { rtnl_addr_set_local(nladdr, laddr_ptr) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }

    // debugging finialized 'addr'
    unsafe {
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_IP,
            format!("addr {:?}", *nladdr),
        )
    };

    // Perform add or remove operations
    let res: c_int;
    match op {
        Operation::Add => {
            // external call to rtnl_addr_add()
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("calling rtnl_addr_add() with nlsock ptr {:?}", nlsock),
            );
            res = unsafe { rtnl_addr_add(nlsock, nladdr, 0) };
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("call to rtnl_addr_add() returned {}", res),
            );
        }
        Operation::Rem => {
            // external call to rtnl_addr_delete()
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("calling rtnl_addr_delete() with nlsock ptr {:?}", nlsock),
            );
            res = unsafe { rtnl_addr_delete(nlsock, nladdr, 0) };
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("call to rtnl_addr_delete() returned {}", res),
            );
        }
    }

    // free allocation of rtnl_addr 'nladdr'
    unsafe { rtnl_addr_put(nladdr) };

    // check 'rtnl_addr_add()|del()' returned value
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

// set_ip_route() function
//
/// Add or delete a route using libnl-3 (netlink)
pub fn set_ip_route(
    _ifindex: i32,
    _ifname: &String,
    route: [u8; 4],
    rtmask: [u8; 4],
    gw: [u8; 4],
    metric: i16,
    mtu: u64,
    op: &Operation,
    debug: &Verbose,
) -> io::Result<()> {
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

    // allocate rtnl_route 'nlroute'
    let nlroute = unsafe { rtnl_route_alloc() };
    if nlroute.is_null() {
        return Err(io::Error::last_os_error());
    }

    // // set ifindex in 'nlroute' if no nh is present
    // unsafe { rtnl_route_set_iif(nlroute, ifindex) };

    // null initialize nl_addr 'rtdst'
    let mut rtdst = nl_addr {
        a_family: 0,
        a_maxsize: 0,
        a_len: 0,
        a_prefixlen: 0,
        a_refcnt: 0,
        a_addr: [0; 4],
    };

    // convert netmask to prefix length
    let mut prefixlen = 0;
    for b in rtmask.iter() {
        prefixlen += b.count_ones();
    }
    // create route string
    let route_str = format!(
        "{}.{}.{}.{}/{}",
        route[0], route[1], route[2], route[3], prefixlen
    );
    // convert route string to a Cstring type
    let route_cstr = CString::new(route_str).unwrap();
    // create pointer to 'nl_addr' rtdst
    let mut rtdst_ptr = &mut rtdst;
    // parse destination route
    let r = unsafe { nl_addr_parse(route_cstr.as_ptr(), AF_INET, &mut rtdst_ptr) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }
    // debug information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        DEBUG_SRC_IP,
        format!(
            "route_slice: {:?}, nl_addr {:?}, result: {}",
            route, *rtdst_ptr, r
        ),
    );
    // set destination route in 'nlroute'
    let r = unsafe { rtnl_route_set_dst(nlroute, rtdst_ptr) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }

    // null initialize nl_addr 'nhaddr'
    let mut nhaddr = nl_addr {
        a_family: 0,
        a_maxsize: 0,
        a_len: 0,
        a_prefixlen: 0,
        a_refcnt: 0,
        a_addr: [0; 4],
    };
    // create nexthop string
    let nh_str = format!("{}.{}.{}.{}", gw[0], gw[1], gw[2], gw[3]);
    // convert nh_str string to a CString
    let nh_cstr = CString::new(nh_str).unwrap();
    // create pointer to 'nl_addr' nhaddr
    let mut nhaddr_ptr = &mut nhaddr;
    // parse Cstring nexthop address in nhaddr
    let r = unsafe { nl_addr_parse(nh_cstr.as_ptr(), AF_INET, &mut nhaddr_ptr) };
    // check for error(s)
    if r < 0 {
        return Err(io::Error::last_os_error());
    }
    // debug information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        DEBUG_SRC_IP,
        format!(
            "gw_slice: {:?}, nl_addr {:?}, result: {}",
            gw, *nhaddr_ptr, r
        ),
    );

    // allocate nexthop
    let rtnh = unsafe { rtnl_route_nh_alloc() };
    if rtnh.is_null() {
        return Err(io::Error::last_os_error());
    }

    // set nexthop's address using 'nhaddr'
    unsafe { rtnl_route_nh_set_gateway(rtnh, nhaddr_ptr) };
    // free nhaddr
    unsafe { nl_addr_put(nhaddr_ptr) };

    // // set nexthop's ifindex
    // // (removed: seems to cause issues on some interfaces)
    // unsafe { rtnl_route_nh_set_ifindex(rtnh, ifindex) };

    // set nexthop in 'nlroute'
    unsafe { rtnl_route_add_nexthop(nlroute, rtnh) };

    // set route metric (possible issue with libl-3) and mtu
    // see 'include/uapi/linux/rtnetlink.h' for RTAX_* types
    let _r = unsafe { rtnl_route_set_metric(nlroute, 32, metric as u32) };
    let r = unsafe { rtnl_route_set_metric(nlroute, 2, mtu as u32) };
    if r < 0 {
        return Err(io::Error::last_os_error());
    }

    // add or remove routes
    let res: c_int;
    match op {
        Operation::Add => {
            // external call to rtnl_route_add()
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("calling rtnl_route_add() with nlsock ptr {:?}", nlsock),
            );
            res = unsafe { rtnl_route_add(nlsock, nlroute, 0) };
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("call to rtnl_route_add() returned {}", res),
            );
        }
        Operation::Rem => {
            // external call to rtnl_route_delete()
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("calling rtnl_route_delete() with nlsock ptr {:?}", nlsock),
            );
            res = unsafe { rtnl_route_delete(nlsock, nlroute, 0) };
            print_debug(
                debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_IP,
                format!("call to rtnl_route_delete() returned {}", res),
            );
        }
    }

    // check 'rtnl_route_add()|delete()' returned value
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    // free nlroute
    unsafe { rtnl_route_put(nlroute) };

    Ok(())
}

// create_macvlan_link() function
//
/// Create a new macvlan interface
pub fn create_macvlan_link(ifindex: i32) -> io::Result<()> {
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

    // allocate macvlan
    let link = unsafe { rtnl_link_macvlan_alloc() };

    // set macvlan master interface to our vr's interface
    unsafe { rtnl_link_set_link(link, ifindex) };

    // initialize ether_vmac
    let vmac_eaddr = int_ether_addr {
        ether_addr: [0, 0, 0, 0, 0, 0],
    };

    // set mac address of macvlan interface to the vr's vmac
    let vmac = unsafe { nl_addr_build(AF_LLC, &vmac_eaddr, ETH_ALEN) };
    unsafe {
        rtnl_link_set_addr(link, vmac);
        nl_addr_put(vmac);
    }

    // set modes on macvlan interface
    let r = unsafe {
        rtnl_link_macvlan_set_mode(link, rtnl_link_macvlan_str2mode("bridge\0".as_bytes()))
    };
    eprintln!("DEBUG: link mode: {}", r);

    // add macvlan link
    let res = unsafe { rtnl_link_add(nlsock, link, INT_NLM_F_CREATE) };

    // check 'rtnl_link_add()|delete()' returned value
    if res < 0 {
        return Err(io::Error::last_os_error());
    }

    // free link object
    unsafe { rtnl_link_put(link) };

    Ok(())
}
