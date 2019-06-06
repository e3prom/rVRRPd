[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/e3prom/rVRRPd/blob/master/LICENSE)
![GitHub top language](https://img.shields.io/github/languages/top/e3prom/rvrrpd.svg)
[![stability-unstable](https://img.shields.io/badge/stability-unstable-yellow.svg)](https://github.com/e3prom/rVRRPd/releases)
![GitHub issues](https://img.shields.io/github/issues-raw/e3prom/rvrrpd.svg)

# Introduction
[rVRRPd](https://github.com/e3prom/rVRRPd) is an open and secure VRRPv2 implementation written in Rust, a rather new programming language known for its inherent memory-safety and speed.

# Development
This project is still in early development stage and only support the Linux operating system. There is no stable API yet, configuration or even documentation at this time. Please keep in mind that the daemon may not be fully interoperable with standard-compliant network equipments and may exhibit stability or even security issues.

# Features
 * Aimed to be Fast and _Secure_ (see development section above)
 * Multithreaded operation (1 thread per VRRP group/interface pair)
 * Easily configurable using [TOML](https://github.com/toml-lang/toml)
 * Interoperable with [RFC3768](https://tools.ietf.org/html/rfc3768) compliant devices
 * Sniffer mode (-m0)
 * Virtual Router in foreground mode (-m1)
 * Virtual Router in daemon mode (-m2)

# Roadmap
 * Linux Netlink Support
 * OpenBSD Support
 * Objects Tracking
 * Interoperability Testing
 * Privileges Separation

# Requirements
 * A Linux 64-bits kernel (only Linux is supported at this time)
 * [Rust Cargo](https://doc.rust-lang.org/cargo/) (required to compile the development version)
 * One or more Ethernet interface (see [conf/rvrrpd.conf](conf/rvrrpd.conf) for a configuration example)
 * Root privileges (required to put interfaces in promiscuous mode, access raw sockets and for use of ioctls)

# Build and run
To quickly build a development version of [rVRRPd](https://github.com/e3prom/rVRRPd), first make sure you have Rust's [Cargo](https://doc.rust-lang.org/cargo/) installed. The recommended way is to first [install](https://doc.rust-lang.org/cargo/getting-started/installation.html) the latest version of Cargo along with the GNU Compiler Collection (GCC) toolchain.

Then build the project by using the `cargo build --release` command:
```
$ cargo build --release
   Compiling libc v0.2.57
   Compiling autocfg v0.1.4
   Compiling semver-parser v0.7.0
   Compiling rand_core v0.4.0
   Compiling arrayvec v0.4.10
   Compiling byteorder v1.3.1
[...]
   Compiling rVRRPd v0.0.8 (/home/e3prom/rVRRPd)
    Finished release [optimized] target(s) in 31.61s

$ target/release/main
Usage: target/release/main -m0|1|2 [options]

Modes:
    0 = VRRPv2 Sniffer
    1 = VRRPv2 Virtual Router (foreground)
    2 = VRRPv2 Virtual Router (daemon)

Options:
    -h, --help          display help information
    -i, --iface INTERFACE
                        ethernet interface to listen on (sniffer mode)
    -m, --mode MODE     operation modes (see Modes):
                        0(sniffer), 1(foreground), 2(daemon)
    -c, --conf FILE     path to configuration file:
                        (default to /etc/rvrrpd/rvrrpd.conf)
    -d, --debug LEVEL   debugging level:
                        0(none), 1(low), 2(medium), 3(high), 5(extensive)

```

To run a VRRP virtual router, edit the configuration file in `conf/rvrrpd.conf` to reflect your environment:
```Text
verbose = 0
debug = 5
pid = "/var/tmp/rvrrpd.pid"
working_dir = "/var/tmp"
main_log = "/var/tmp/rvrrpd.log"
error_log = "/var/tmp/rvrrpd-error.log"

[[vrouter]]
group = 1
interface = "ens192.900"
vip = "10.100.100.1"
priority = 254
preemption = true
auth_type = 0
#auth_secret = ""
#timers = { advert = 1 }

[protocols]
    [[protocols.static]]
    route = "0.0.0.0"
    mask = "0.0.0.0"
    nh = "10.240.0.254"
```
The above configuration example do the following:
 * Starts the daemon in foreground with a debug level of 5 (extensive).
 * Runs one virtual-router with group id `1` on interface `ens192.900`, with the below parameters:
   * Uses the virtual IP address `10.100.100.1`.
   * Is configured with the highest priority of `254`.
   * Has preeemption enabled.
   * Is configured without authentication (auth type 0).
* Has a static defaut route configured with a next-hop of `10.240.0.254`.

Finally run the binary executable `main` you just built using the command-line parameter `-m1`, to enter the `Virtual Router (foreground)` operating mode:
```
$ sudo target/release/main -m1
Starting rVRRPd
[...]
```

Your virtual router will now listen for VRRPv2 packets and will take the Master or Backup role when necessary. If the router own the virtual IP address, it will automatically take the Master role with a priority of 255.

# Support
If you are experiencing any stability, security or interoperability problems, feel free to open a [new issue](https://github.com/e3prom/rVRRPd/issues/new). Please provide a full backtrace in case the daemon panics or crashes.
