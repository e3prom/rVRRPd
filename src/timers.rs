//! Timers related functions module
//! This module implements the various timers threads using tokio.
use super::*;

// tokio
use tokio::prelude::*;
use tokio::timer::Interval;

// futures
use futures::Future;

// channels
use std::sync::mpsc;

// debugging
use crate::debug::{print_debug, Verbose};

// fsm
use crate::fsm::Event;

// std
use std::sync::RwLock;
use std::time::{Duration, Instant};

// start_timers() function
/// starts the various timers needed for the protocol and internal operations
pub fn start_timers(
    tx: Arc<Mutex<mpsc::Sender<Event>>>,
    vr: Arc<RwLock<VirtualRouter>>,
    debug: &Verbose,
) {
    // clone vr's Arc
    let vr0 = Arc::clone(&vr);
    // acquire read lock on virtual router
    let vr0 = vr0.read().unwrap();

    // clone debug
    let debug = debug.clone();

    // set duration from vr's timer
    let master_down = Duration::from_secs(vr0.timers.master_down() as u64);
    let advert = Duration::from_secs(vr0.timers.advert() as u64);

    // drop the lock as we don't need read access to vr anymore
    drop(vr0);

    // clone tx channel
    let tx1 = Arc::clone(&tx);
    // clone the vr's Arc
    let vr1 = Arc::clone(&vr);
    let vr2 = Arc::clone(&vr);

    // new instance of 'master_down' interval
    // this is a countdown-type timer which verify that at least one ADVERTISEMEMNT
    // packet has been received since the last call of the timer. If the flag equal
    // 0x1, no ADVERTISEMENT has been received (since) and the master is signaled
    // down to the approriate vr's thread, this timer share the 'tx' channel with.
    let master_down_int = Interval::new_interval(master_down)
        .take_while(move |_| future::ok(is_master_down_disabled(&vr1, &debug)))
        .for_each(move |_| {
            print_debug(
                &debug,
                DEBUG_LEVEL_HIGH,
                DEBUG_SRC_TIMER,
                format!("master_down interval has expired"),
            );

            // first acquire read lock on vr
            let vr2 = vr2.read().unwrap();

            // if flag is already set, then signal master down
            if vr2.flags.get_down_flag() == 0x1 {
                print_debug(
                    &debug,
                    DEBUG_LEVEL_EXTENSIVE,
                    DEBUG_SRC_TIMER,
                    format!("signaling Master down"),
                );
                // acquire transmit channel lock
                let tx1 = tx1.lock().unwrap();
                // send MasterDown Event down the channel
                tx1.send(Event::MasterDown).unwrap();
                // return Ok(())
                Ok(())
            }
            // check if down flag is set
            else if vr2.flags.get_down_flag() == 0x0 {
                print_debug(
                    &debug,
                    DEBUG_LEVEL_EXTENSIVE,
                    DEBUG_SRC_TIMER,
                    format!("signaling master_down timer expiry"),
                );
                // acquire transmit channel lock
                let tx1 = tx1.lock().unwrap();
                // send MasterDown Event down the channel
                tx1.send(Event::MasterDownExpiry).unwrap();
                // return Ok(())
                Ok(())
            } else {
                Ok(())
            }
        })
        .map_err(|_| ());

    // clone transmit channel
    let tx2 = Arc::clone(&tx);
    // clone the vr's Arc
    let vr3 = Arc::clone(&vr);

    // new advertisement interval-type timer
    // when this future executes, a GenAdvert notification is sent to the worker thread
    // which then trigger an ADVERTISEMENT message in some finite state machine states.
    let advert_int = Interval::new(Instant::now() + advert, advert)
        // must return true to activate the interval timer
        .take_while(move |_| future::ok(is_advert_disabled(&vr3, &debug)))
        .for_each(move |_| {
            // print debugging information
            print_debug(
                &debug,
                DEBUG_LEVEL_HIGH,
                DEBUG_SRC_TIMER,
                format!("advertisement interval has expired"),
            );
            // acquire lock on transmit channel
            let tx2 = tx2.lock().unwrap();
            // print debugging information
            print_debug(
                &debug,
                DEBUG_LEVEL_EXTENSIVE,
                DEBUG_SRC_TIMER,
                format!("signaling advertisement interval expiry"),
            );
            // send GenAdvert event to worker thread
            tx2.send(Event::GenAdvert).unwrap();
            // return Ok(())
            Ok(())
        })
        .map_err(|_| ());

    // start the tokio runtime
    tokio::run(future::lazy(|| {
        tokio::spawn(master_down_int);
        tokio::spawn(advert_int);
        Ok(())
    }));
}

// is_master_down_disabled() function
/// return boolean false is the master_down interval is zero or lower
fn is_master_down_disabled(vr: &Arc<RwLock<VirtualRouter>>, debug: &Verbose) -> bool {
    let vr = vr.read().unwrap();
    if vr.timers.master_down() > 0.0 {
        true
    } else {
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_MEDIUM,
            DEBUG_SRC_TIMER,
            format!("master_down timer is disabled"),
        );
        false
    }
}

// is_advert_disabled() function
/// return boolean true is the advertisement interval vr's timer
/// is higher than zero
fn is_advert_disabled(vr: &Arc<RwLock<VirtualRouter>>, debug: &Verbose) -> bool {
    let vr = vr.read().unwrap();
    if vr.timers.advert() > 0 {
        true
    } else {
        // print debugging information
        print_debug(
            debug,
            DEBUG_LEVEL_MEDIUM,
            DEBUG_SRC_TIMER,
            format!("the advertisement timer is disabled"),
        );
        false
    }
}
