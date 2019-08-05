[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/e3prom/rVRRPd/blob/master/LICENSE)
![GitHub top language](https://img.shields.io/github/languages/top/e3prom/rvrrpd.svg)
![GitHub issues](https://img.shields.io/github/issues/e3prom/rvrrpd.svg)
![GitHub last commit](https://img.shields.io/github/last-commit/e3prom/rvrrpd.svg)
[![Build Status](https://travis-ci.org/e3prom/rVRRPd.svg?branch=master)](https://travis-ci.org/e3prom/rVRRPd)

# Introduction
[rVRRPd](https://github.com/e3prom/rVRRPd) is an open and standard-compliant VRRPv2 implementation written in [Rust](https://www.rust-lang.org/), a programming language known for its inherent portability, memory-safety and speed.

# Features
 * Aimed to be Robust, Fast and Secure
 * Multi-threaded operation (1 thread per VRRP group/interface pair)
 * Easily configurable using [TOML](https://github.com/toml-lang/toml) or [JSON](https://www.json.org/)
 * Interoperable with [`RFC3768`](https://tools.ietf.org/html/rfc3768) (VRRPv2) compliant devices
 * Authentication Support
   * Password Authentication (Type-1) based on [`RFC2338`](https://tools.ietf.org/html/rfc2338) 
   * Proprietary P0 HMAC (based on SHA256 truncated to 8 bytes)
   * Proprietary P1 (SHAKE256 XOF)
 * Support multiple operating modes:
   * Sniffer mode (`-m0`)
   * Virtual Router in foreground mode (`-m1`)
   * Virtual Router in daemon mode (`-m2`)
 * MAC-based Virtual LAN interface (`macvlan`) support

# Development
This project is still in **_early development_** stage, and at this time, only supports the Linux operating system. There is no stable API, configuration or even documentation yet. Please keep in mind that [`rVRRPd`](https://github.com/e3prom/rVRRPd) may not be fully interoperable with standard-compliant network equipments and may also exhibit stability issues, therefore use at your own risks.

## Roadmap
The current development roadmap for version 0.2.0 can be found [here](https://github.com/e3prom/rVRRPd/projects/2).

# Dependencies
 * A Linux 64-bits kernel (only Linux is currently supported).
 * Rust's [`Cargo`](https://doc.rust-lang.org/cargo/), which is not required but recommended to easily compile this project and all its dependencies.
 * One or more Ethernet interface(s), see [`conf/rvrrpd.conf`](conf/rvrrpd.conf) for a basic configuration example.
 * Root privileges, required to put interfaces in promiscuous mode and access raw sockets.
 * The [`libnl`](https://www.infradead.org/~tgr/libnl/) library for accessing the netlink interface on Linux.

# Build and run
To quickly build a development version of [`rVRRPd`](https://github.com/e3prom/rVRRPd), first make sure you have the latest version of [`Cargo`](https://doc.rust-lang.org/cargo/) installed. The recommended steps are to first [install](https://doc.rust-lang.org/cargo/getting-started/installation.html) Cargo, then the GNU Compiler Collection (GCC) toolchain and lastly the `libnl-3` development packages (including headers files), namely `libnl-3-dev` and `libnl-route-3-dev` on Debian and derivatives.

To quickly build the daemon executable, use the `cargo build --release` command:
```console
$ cargo build --release
   Compiling ryu v1.0.0
   Compiling itoa v0.4.4
   Compiling ctrlc v3.1.2
   Compiling serde_json v1.0.40
   Compiling rVRRPd v0.1.0 (/home/e3prom/rVRRPd)
    Finished release [optimized] target(s) in 9.63s

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
    -g, --cfg-format FORMAT
                        configuration format: toml(default), json
```

Before running the VRRPv2 daemon, copy the example configuration file at [`conf/rvrrpd.conf`](conf/rvrrpd.conf) to the default configuration file path `/etc/rvrrpd/rvrrpd.conf`. Then use your favorite text editor to configure the virtual router(s) to your needs.

See below for an example of a virtual router running on an Ethernet interface with password authentication and preemption enabled:
```TOML
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
rfc3768 = true
netdrv = "libnl"
auth_type = "rfc2338-simple"
auth_secret = "thissecretnolongeris"

[protocols]
    [[protocols.static]]
    route = "0.0.0.0"
    mask = "0.0.0.0"
    nh = "10.240.0.254"

```
The above configuration do the following:
 * Starts the daemon in foreground mode with a debug level of 5 (extensive).
 * Runs one virtual-router with group id `1` on interface `ens192.900`, with the below parameters:
   * Uses the virtual IP address `10.100.100.1`.
   * Is configured with the highest priority of `254`.
   * Has preeemption enabled.
   * Has compatibility with [`RFC3768`](https://tools.ietf.org/html/rfc3768) turned on (may be required to fully interoperate with some vendors).
   * Uses the network driver `libnl` which leverage the netlink protocol. Alternatively, you can use the `ioctl` driver, which is simpler but will removes the interface's IP addresse(s) for the VIP when in Master state.
   * Set authentication to the [`RFC2338`]'s (https://tools.ietf.org/html/rfc2338) `Simple Password` authentication method.
   * Set the secret key (or password) to be shared between the virtual routers.
* When master, install a static default route with a next-hop of `10.240.0.254`.

Finally run the binary executable `main` you just built using the command-line parameter `-m1`, to start the daemon in foreground mode:
```bash
$ sudo target/release/main -m1 -c conf/rvrrpd.conf
Starting rVRRPd
[...]
```

Your virtual router will now listen for VRRPv2 packets and will take the `Master` or `Backup` role. If the router owns the virtual IP address, it will automatically take the `Master` role with a priority of `255`.

# Support
If you are experiencing any stability, security or interoperability problems, feel free to open a [new issue](https://github.com/e3prom/rVRRPd/issues/new).
