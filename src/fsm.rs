//! finite-state machine module
//! This module includes the finite-state machine (FSM).
use super::*;

// channels and threads
use std::sync::{Arc, Mutex};

// threads
use std::thread;

// debugging
use crate::debug::Verbose;

// operating system drivers
use crate::os::drivers::Operation;

// address resolution protocol
#[cfg(target_os = "linux")]
use os::linux::arp::open_raw_socket_arp;

/// Internal Protocol States "Enumerator"
#[derive(Debug)]
pub enum States {
    Down,   // Special down state
    Init,   // Initialize
    Backup, // Virtual Router is Backup
    Master, // Virtual Router is Master
}

/// Timers Structure
#[derive(Debug)]
pub struct Timers {
    master_down: f32, // Timer that fires when ADVERTISEMENT has not been heared for 'master_down'
    advert: u8,       // Timer that fires sending of ADVERTISEMENT every 'advertint' interval
}

// Timers Type Implementation
impl Timers {
    // new() method
    pub fn new(master_down: f32, advert: u8) -> Timers {
        Timers {
            master_down,
            advert,
        }
    }
    // master_down() getter
    pub fn master_down(&self) -> f32 {
        self.master_down
    }
    // advert() getter
    pub fn advert(&self) -> u8 {
        // make sure the advertisement timer is higher than zero
        //assert!(self.advert > 0);
        self.advert
    }
}

/// Flags Structure
#[derive(Debug)]
pub struct Flags {
    down: u8,
}

// Flags Type Implementation
impl Flags {
    // new() method
    pub fn new(down: u8) -> Flags {
        Flags { down }
    }
    // get_down_flag() method
    pub fn get_down_flag(&self) -> u8 {
        self.down
    }
    // set_down_flag() method
    pub fn set_down_flag(&mut self) {
        self.down = 0x1;
    }
    // clear_down_flag() method
    pub fn clear_down_flag(&mut self) {
        self.down = 0x0;
    }
}

/// Event Enumerator
#[derive(Debug)]
pub enum Event {
    Startup,
    Shutdown, // Internal Shutdown Event
    Terminate,
    MasterDown,          // internal master down notification
    MasterDownExpiry,    // internal master_down timer expiry notification
    Advert([u8; 4], u8), // got ADVERTISEMENT message (carrying priority)
    GenAdvert,           // generate an ADVERTISEMENT message
}

// fsm_run() function
/// run the finite-state machine (FSM)
pub fn fsm_run(
    id: usize,
    tx: &Arc<Mutex<mpsc::Sender<Event>>>,
    rx: &Arc<Mutex<mpsc::Receiver<Event>>>,
    vr: &Arc<RwLock<VirtualRouter>>,
    fd: i32,
    debug: &Verbose,
) {
    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_HIGH,
        DEBUG_SRC_FSM,
        format!("registering notification sender channel for thread {}", id),
    );

    // register notification sender channel
    register_tx(&vr, &tx, id, &debug);

    // start thread loop
    loop {
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_WORKER,
            format!("worker thread {} acquiring lock on rx channel", id),
        );
        // acquire lock on receive channel
        let event = rx.lock().unwrap();

        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_WORKER,
            format!("worker thread {} waiting for events", id),
        );
        // listen for events
        let event = event.recv().unwrap();
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_WORKER,
            format!("worker thread {} got new {:?} event", id, event),
        );

        // handle terminate event first and foremost
        match event {
            // got Termination event
            Event::Terminate => {
                // print debugging information
                print_debug(
                    debug,
                    DEBUG_LEVEL_HIGH,
                    DEBUG_SRC_WORKER,
                    format!("worker thread {} exited", id),
                );
                // break current loop
                break;
            }
            _ => {}
        }

        // clone tx channel and vr for timer thread(s)
        let timer_tx = Arc::clone(&tx);
        let timer_vr = Arc::clone(&vr);

        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_WORKER,
            format!("worker thread {} acquiring write lock", id),
        );
        // acquire write lock on thread's virtual router
        let mut vr = vr.write().unwrap();
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_WORKER,
            format!("worker thread {} write lock acquired", id),
        );

        // evaluate virtual router's current state
        let st = match &vr.get_states() {
            States::Down => {
                continue;
            }
            States::Init => {
                match event {
                    // event: if a Startup event is received
                    Event::Startup => {
                        // print debugging information
                        print_debug(
                            debug,
                            DEBUG_LEVEL_EXTENSIVE,
                            DEBUG_SRC_WORKER,
                            format!("got Startup event on thread {}", id),
                        );

                        // print information
                        let vip = vr.parameters.vip();
                        print_debug(debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                            "Starting VRRP Virtual Router ({}.{}.{}.{}) for group {}, on interface {} (thread: {})",
                            vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface(), id
                        ));

                        // print debugging information
                        print_debug(
                            debug,
                            DEBUG_LEVEL_EXTENSIVE,
                            DEBUG_SRC_WORKER,
                            format!("starting timer thread from worker thread {}", id),
                        );

                        // starting timer thread(s)
                        // and clone debug structure of type Verbose
                        let d = debug.clone();
                        let _timer_thread = thread::spawn(move || {
                            timers::start_timers(timer_tx, timer_vr, &d);
                        });

                        // if the virtual router is the owner of the virtual ip address
                        // OR the priority has been configured at 255
                        if vr.is_owner_vip(&vr.parameters.vip()) || vr.parameters.prio() == 255 {
                            // force the priority to 255
                            vr.parameters.set_prio(255);
                            // set VRRP virtual mac address
                            let mut vmac = ETHER_VRRP_V2_SRC_MAC;
                            vmac[5] = vr.parameters.vrid();

                            // --- Linux specific interface tyoe handling
                            #[cfg(target_os = "linux")]
                            {
                                // setup MAC address or virtual interface
                                match vr.parameters.iftype() {
                                    // if vr's interface is of type macvlan
                                    IfTypes::macvlan => {
                                        // create macvlan interface
                                        match vr.setup_macvlan_link(vmac, Operation::Add, debug) {
                                            Some((vif_idx, vif_name)) => {
                                                // store the virtual interface's index
                                                vr.parameters.set_vifidx(vif_idx);
                                                // save master interface to vif_name
                                                let vif = vr.parameters.interface();
                                                vr.parameters.set_vifname(vif);
                                                // change current vr's interface to the virtual interface
                                                vr.parameters.set_interface(vif_name);
                                                // save vif interface mac
                                                let ifmac = vr.get_mac_addresses(fd, debug);
                                                vr.parameters.set_ifmac(ifmac);
                                            }
                                            // if it failed for some reasons, do not change vr's interface
                                            None => (),
                                        };
                                    }
                                    _ => {
                                        // save vr's interface mac (old)
                                        let ifmac = vr.get_mac_addresses(fd, debug);
                                        vr.parameters.set_ifmac(ifmac);
                                        // set virtual router's MAC address
                                        vr.set_mac_addresses(fd, vmac, debug);
                                    }
                                }
                            }
                            // END Linux specific interface type handling

                            // send an ADVERTISEMENT message
                            match vr.send_advertisement(fd, &debug) {
                                Ok(_) => (),
                                Err(e) => eprintln!(
                                    "error(fsm): error while sending VRRP advertisement on interface {}: {}",
                                    vr.parameters.interface(),
                                    e
                                ),
                            }

                            // --- Linux specific ARP handling
                            #[cfg(target_os = "linux")]
                            {
                                // send gratuitious ARP requests
                                let arp_sockfd = open_raw_socket_arp().unwrap();
                                vr.broadcast_gratuitious_arp(arp_sockfd, debug).unwrap();
                            }
                            // END Linux specific ARP handling

                            // --- FreeBSD specific ARP handling
                            #[cfg(target_os = "freebsd")]
                            {
                                // reuse BPF file descriptor
                                vr.broadcast_gratuitious_arp(fd, &debug).unwrap();
                            }
                            // END FreeBSD specific ARP handling

                            // set advertisement interval
                            vr.timers.advert = vr.parameters.adverint();
                            // print debugging information
                            print_debug(
                                &debug,
                                DEBUG_LEVEL_EXTENSIVE,
                                DEBUG_SRC_FSM,
                                format!("the advertisement interval is now {}s", vr.timers.advert),
                            );
                            // print information
                            let vip = vr.parameters.vip();
                            print_debug(&debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                                "VR {}.{}.{}.{} for group {} on interface {} - Changed from Init to Master",
                                vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface()
                            ));
                            // transition to Master state
                            fsm::States::Master
                        } else {
                            // set master_down timer
                            vr.timers.master_down = vr.parameters.master_down();
                            // print information
                            let vip = vr.parameters.vip();
                            print_debug(&debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                                "VR {}.{}.{}.{} for group {} on interface {} - Changed from Init to Backup",
                                vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface()
                            ));
                            // transition to Backup state
                            States::Backup
                        }
                    }
                    // event: if Shutdown event is received
                    Event::Shutdown => {
                        // print information
                        let vip = vr.parameters.vip();
                        print_debug(&debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                            "VR {}.{}.{}.{} for group {} on interface {} - Changed from Init to Down",
                            vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface()
                        ));
                        // transition to Down state
                        States::Down
                    }
                    _ => {
                        // print debugging information
                        print_debug(
                            debug,
                            DEBUG_LEVEL_EXTENSIVE,
                            DEBUG_SRC_FSM,
                            format!("unexpected event catched in Init state"),
                        );
                        continue;
                    }
                }
            }
            States::Backup => {
                match event {
                    // event: If ADVERTISEMENT message is received
                    Event::Advert(_ipsrc, prio) => {
                        // if the priority is zero then set the master_down timer to skew_time
                        if prio == 0 {
                            // set master_down interval to skew_time (necessary?)
                            vr.timers.master_down = vr.parameters.skewtime();
                        } else {
                            // if priority is greater than or equal to the local priority OR preempt is false
                            if vr.parameters.preempt() == false || prio >= vr.parameters.prio() {
                                // reset master_down interval (necessary?)
                                vr.timers.master_down = vr.parameters.master_down();
                                // clear down flag (signal master is alive)
                                vr.flags.clear_down_flag();
                                // print debugging information
                                print_debug(
                                    debug,
                                    DEBUG_LEVEL_HIGH,
                                    DEBUG_SRC_FSM,
                                    format!("down flag cleared in Backup state"),
                                );
                            }
                        }
                        continue;
                    }
                    // event: master_down timer has expired
                    Event::MasterDownExpiry => {
                        // set down flag
                        vr.flags.set_down_flag();
                        // print debugging information
                        print_debug(
                            debug,
                            DEBUG_LEVEL_HIGH,
                            DEBUG_SRC_FSM,
                            format!("down flag set for worker thread {}", id),
                        );
                        continue;
                    }
                    // event: If the Timers::master_down reached zero
                    Event::MasterDown => {
                        // print information
                        let vip = vr.parameters.vip();
                        print_debug(
                            debug,
                            DEBUG_LEVEL_INFO,
                            DEBUG_SRC_INFO,
                            format!(
                                "VR {}.{}.{}.{} for group {} on interface {} - Master VR is down",
                                vip[0],
                                vip[1],
                                vip[2],
                                vip[3],
                                vr.parameters.vrid(),
                                vr.parameters.interface()
                            ),
                        );
                        // set VRRP virtual mac address
                        let mut vmac = ETHER_VRRP_V2_SRC_MAC;
                        vmac[5] = vr.parameters.vrid();

                        // --- Linux specific interface type handling
                        #[cfg(target_os = "linux")]
                        // setup MAC address or virtual interface
                        match vr.parameters.iftype() {
                            // if vr's interface is of type macvlan
                            IfTypes::macvlan => {
                                // create macvlan interface
                                match vr.setup_macvlan_link(vmac, Operation::Add, debug) {
                                    Some((vif_idx, vif_name)) => {
                                        // store the virtual interface's index
                                        vr.parameters.set_vifidx(vif_idx);
                                        // save master interface to vif_name
                                        let phys = vr.parameters.interface();
                                        vr.parameters.set_vifname(phys);
                                        // change current vr's interface to the virtual interface
                                        vr.parameters.set_interface(vif_name);
                                        // save vif interface mac
                                        let ifmac = vr.get_mac_addresses(fd, debug);
                                        vr.parameters.set_ifmac(ifmac);
                                    }
                                    // if it failed for some reasons, do not change vr's interface
                                    None => (),
                                };
                            }
                            _ => {
                                // save vr's interface mac (old)
                                let ifmac = vr.get_mac_addresses(fd, debug);
                                vr.parameters.set_ifmac(ifmac);
                                // set virtual router's MAC address
                                vr.set_mac_addresses(fd, vmac, debug);
                            }
                        }
                        // END Linux specific interface type handling

                        // --- Linux specific interface type handling
                        #[cfg(target_os = "linux")]
                        // set VIP according to network driver in use
                        match vr.parameters.netdrv() {
                            NetDrivers::ioctl => {
                                // set IP addresses (including VIP) on the vr's interface
                                vr.set_ip_addresses(fd, Operation::Add, debug);
                                // set routes
                                vr.set_ip_routes(fd, Operation::Add, debug);
                            }
                            NetDrivers::libnl => {
                                // add vip on vr's interface
                                vr.set_ip_addresses(fd, Operation::Add, debug);
                                // set routes
                                vr.set_ip_routes(fd, Operation::Add, debug);
                            }
                        }
                        // END Linux specific interface type handling

                        // --- Linux specific ARP handling
                        #[cfg(target_os = "linux")]
                        {
                            // send gratuitious ARP requests
                            let arp_sockfd = open_raw_socket_arp().unwrap();
                            vr.broadcast_gratuitious_arp(arp_sockfd, debug).unwrap();
                        }
                        // END Linux specific ARP handling

                        // --- FreeBSD specific interface tyoe handling
                        #[cfg(target_os = "freebsd")]
                        {
                            // set VIP
                            vr.set_ip_addresses(fd, Operation::Add, debug);
                            // reuse BPF file descriptor
                            vr.broadcast_gratuitious_arp(fd, &debug).unwrap();
                        }
                        // END FreeBSD specific interface tyoe handling

                        // set advertisement timer
                        vr.timers.advert = vr.parameters.adverint();
                        // send ADVERTISEMENT
                        match vr.send_advertisement(fd, &debug) {
                            Ok(_) => (),
                            Err(e) => eprintln!(
                                "error(fsm): error while sending VRRP advertisement on interface {}: {}",
                                vr.parameters.interface(),
                                e
                            ),
                        }
                        // print information
                        let vip = vr.parameters.vip();
                        print_debug(&debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                            "VR {}.{}.{}.{} for group {} on interface {} - Changed from Backup to Master",
                            vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface()
                        ));
                        // transition to Master state
                        States::Master
                    }
                    // event: if Shutdown event is received
                    Event::Shutdown => {
                        // print information
                        let vip = vr.parameters.vip();
                        print_debug(&debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                            "VR {}.{}.{}.{} for group {} on interface {} - Changed from Backup to Down",
                            vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface()
                        ));
                        // cancel the 'active' master_down timer
                        vr.timers.master_down = std::f32::NAN;
                        // transition to Down state
                        States::Down
                    }
                    _ => {
                        continue;
                    }
                }
            }
            States::Master => {
                match event {
                    // event: Advertisement timer expired in timer thread
                    Event::GenAdvert => {
                        // send ADVERTISEMENT message
                        match vr.send_advertisement(fd, &debug) {
                            Ok(_) => (),
                            Err(e) => eprintln!(
                                "error(fsm): error while sending VRRP advertisement on interface {}: {}",
                                vr.parameters.interface(),
                                e
                            ),
                        }
                        // reset the advertisement timer to advertisement interval
                        vr.timers.advert = vr.parameters.adverint();
                        continue;
                    }
                    // event: we got an ADVERTISEMENT message
                    Event::Advert(ipsrc, prio) => {
                        // if priority is zero
                        if prio == 0 {
                            // send an ADVERTISEMENT message
                            match vr.send_advertisement(fd, &debug) {
                                Ok(_) => (),
                                Err(e) => eprintln!(
                                    "error(fsm): error while sending VRRP advertisement on interface {}: {}",
                                    vr.parameters.interface(),
                                    e
                                ),
                            }
                            // reset the advertisement timer to advertisement interval
                            vr.timers.advert = vr.parameters.adverint();
                            // state doesn't change
                            continue;
                        } else {
                            // if ADVERTISEMENT priority is greater than local priority
                            // OR (the priority is equal AND primary address is higher than
                            // local address)
                            if prio > vr.parameters.prio()
                                || (prio == vr.parameters.prio()
                                    && is_primary_higher(&ipsrc, &vr.parameters.ipaddrs()[0]))
                            {
                                // cancel advertisement timer
                                vr.timers.advert = 0;
                                // reset master_down timer to master_down interval
                                vr.timers.master_down = vr.parameters.master_down();
                                // clear down flag (mark master alive)
                                vr.flags.clear_down_flag();
                                // print debugging information
                                print_debug(
                                    debug,
                                    DEBUG_LEVEL_HIGH,
                                    DEBUG_SRC_FSM,
                                    format!("down flag cleared in Master state"),
                                );

                                // --- Linux specific interface tyoe handling
                                #[cfg(target_os = "linux")]
                                // restore primary or delete vip on vr's interface
                                match vr.parameters.iftype() {
                                    IfTypes::macvlan => {
                                        // removes macvlan interface
                                        vr.setup_macvlan_link(
                                            vr.parameters.ifmac(),
                                            Operation::Rem,
                                            debug,
                                        );
                                        // restore back vif and physical interfaces
                                        let vif = vr.parameters.interface();
                                        let phys = vr.parameters.vifname();
                                        vr.parameters.set_vifname(vif);
                                        vr.parameters.set_interface(phys);
                                        // remove added routes
                                        vr.set_ip_routes(fd, Operation::Rem, debug);
                                    }
                                    _ => {
                                        // restore interface's MAC address
                                        vr.set_mac_addresses(fd, vr.parameters.ifmac(), debug);
                                        match vr.parameters.netdrv() {
                                            NetDrivers::ioctl => {
                                                // restore primary IP
                                                #[cfg(target_os = "linux")]
                                                vr.set_ip_addresses(fd, Operation::Rem, debug);
                                                // re-set routes
                                                vr.set_ip_routes(fd, Operation::Add, debug);
                                            }
                                            NetDrivers::libnl => {
                                                // delete vip
                                                vr.delete_ip_addresses(fd, debug);
                                            }
                                        }
                                    }
                                }
                                // END Linux specific interface type handling

                                // -- FreeBSD specific interface type handling
                                #[cfg(target_os = "freebsd")]
                                {
                                    // we don't have to re-set the mac address here
                                    // delete the VIP
                                    vr.delete_ip_addresses(fd, debug);
                                }
                                // END FreeBSD specific interface type handling

                                // print information
                                let vip = vr.parameters.vip();
                                print_debug(&debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                                    "VR {}.{}.{}.{} for group {} on interface {} - Changed from Master to Backup",
                                    vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface()
                                ));
                                // transition to Backup state
                                States::Backup
                            } else {
                                continue;
                            }
                        }
                    }
                    // event: master is declared down by timer thread
                    Event::MasterDown => {
                        // print debugging information
                        print_debug(
                            debug,
                            DEBUG_LEVEL_EXTENSIVE,
                            DEBUG_SRC_FSM,
                            format!("received MasterDown event in Master State"),
                        );
                        continue;
                    }
                    // event: if Shutdown event is received
                    Event::Shutdown => {
                        // print information
                        let vip = vr.parameters.vip();
                        print_debug(&debug, DEBUG_LEVEL_INFO, DEBUG_SRC_INFO, format!(
                            "VR {}.{}.{}.{} for group {} on interface {} - Changed from Master to Down",
                            vip[0], vip[1], vip[2], vip[3], vr.parameters.vrid(), vr.parameters.interface()
                        ));
                        // cancel the 'advert' timer
                        vr.timers.advert = 0;
                        // send ADVERTISEMENT with priority equal 0
                        vr.parameters.set_prio(0);
                        match vr.send_advertisement(fd, &debug) {
                            Ok(_) => (),
                            Err(e) => eprintln!(
                                "error(fsm): error while sending VRRP advertisement on interface {}: {}",
                                vr.parameters.interface(),
                                e
                            ),
                        }

                        // -- Linux specific interface tyoe handling
                        #[cfg(target_os = "linux")]
                        match vr.parameters.iftype() {
                            IfTypes::macvlan => {
                                // removes macvlan interface
                                vr.setup_macvlan_link(vr.parameters.ifmac(), Operation::Rem, debug);
                                // restore back vif and physical interfaces
                                let vif = vr.parameters.interface();
                                let phys = vr.parameters.vifname();
                                vr.parameters.set_vifname(vif);
                                vr.parameters.set_interface(phys);
                                // remove routes
                                vr.set_ip_routes(fd, Operation::Rem, debug);
                            }
                            _ => {
                                // restore interface's MAC address
                                vr.set_mac_addresses(fd, vr.parameters.ifmac(), debug);
                                // restore primary or delete vip on vr's interface
                                match vr.parameters.netdrv() {
                                    NetDrivers::ioctl => {
                                        // restore primary IP
                                        vr.set_ip_addresses(fd, Operation::Rem, debug);
                                        // remove routes
                                        vr.set_ip_routes(fd, Operation::Rem, debug);
                                    }
                                    NetDrivers::libnl => {
                                        // delete vip
                                        vr.delete_ip_addresses(fd, debug);
                                        // remove added routes
                                        vr.set_ip_routes(fd, Operation::Rem, debug);
                                    }
                                }
                            }
                        }
                        // END Linux specific interface type handling

                        // -- FreeBSD specific interface type handling
                        #[cfg(target_os = "freebsd")]
                        {
                            // we don't have to re-set the mac address here
                            // delete the VIP
                            vr.delete_ip_addresses(fd, debug);
                        }
                        // END FreeBSD specific interface type handling

                        // transition to Down state
                        States::Down
                    }
                    _ => {
                        continue;
                    }
                }
            }
        };
        // set end-of-loop state
        vr.set_states(st);
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_EXTENSIVE,
            DEBUG_SRC_WORKER,
            format!("worker thread {} released locks", id),
        );
    }
}

// register_tx() function
/// registers the virtual router sending channel
fn register_tx(
    vr: &Arc<RwLock<VirtualRouter>>,
    tx: &Arc<Mutex<mpsc::Sender<Event>>>,
    id: usize,
    debug: &Verbose,
) {
    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        DEBUG_SRC_WORKERG,
        format!("acquiring write lock for thread {}", id),
    );
    // acquire write lock on virtual router
    let mut vr = vr.write().unwrap();
    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        DEBUG_SRC_WORKERG,
        format!("acquired write lock for thread {}", id),
    );

    // setting up the notification tx channel
    vr.parameters.set_notification(Arc::clone(tx));
    // print debugging information
    print_debug(
        debug,
        DEBUG_LEVEL_EXTENSIVE,
        DEBUG_SRC_WORKERG,
        format!("registered tx channel for thread {}", id),
    );
}

// is_primary_higher() function
/// return a boolean true if the primary address is higher than local
fn is_primary_higher(primary: &[u8; 4], local: &[u8; 4]) -> bool {
    let mut result = false;
    // do a per byte comparison, and returns true if primary's is higher
    for (i, b) in primary.iter().enumerate() {
        if *b > local[i] {
            result = true;
        }
    }
    result
}
