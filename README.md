[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/e3prom/rVRRPd/blob/master/LICENSE)
![GitHub top language](https://img.shields.io/github/languages/top/e3prom/rvrrpd.svg)
![GitHub issues](https://img.shields.io/github/issues/e3prom/rvrrpd.svg)
![GitHub last commit](https://img.shields.io/github/last-commit/e3prom/rvrrpd.svg)
[![Build Status](https://travis-ci.org/e3prom/rVRRPd.svg?branch=master)](https://travis-ci.org/e3prom/rVRRPd)

# Introduction
[rVRRPd](https://github.com/e3prom/rVRRPd) is an open and standard-compliant VRRPv2 implementation written in [Rust](https://www.rust-lang.org/), a programming language known for its inherent portability, memory-safety and speed.

# Features
 * Aimed to be Portable, Fast and **Secure**
 * Supports Intel IA-64 and ARMv8 Processors
 * Multi-threaded operation (1 thread per VRRP group/interface pair)
 * Easily configurable using [TOML](https://github.com/toml-lang/toml) or [JSON](https://www.json.org/) formats
 * Interoperable with [`RFC3768`](https://tools.ietf.org/html/rfc3768) (VRRPv2) compliant devices
 * Authentication Support
   * Password Authentication (Type-1) based on [`RFC2338`](https://tools.ietf.org/html/rfc2338)
   * Proprietary P0 HMAC (based on SHA256 truncated to 8 bytes)
   * Proprietary P1 (SHAKE256 XOF)
 * Supports multiple operation modes:
   * Sniffer mode (`-m0`)
   * Virtual Router in foreground mode (`-m1`)
   * Virtual Router in daemon mode (`-m2`)
 * Support MAC-based Virtual LAN interface (`macvlan`)

# Development
This project is still in **_development_** stage, and at this time, only supports the Linux operating system. There is no stable API, configuration or even documentation yet. Please keep in mind that [`rVRRPd`](https://github.com/e3prom/rVRRPd) may not be fully interoperable with standard-compliant network equipments.

The development roadmap for the upcoming `0.2.0` release can be found on its [dedicated project page](https://github.com/e3prom/rVRRPd/projects/2).

# Dependencies
 * A Linux 64-bits kernel (_only Linux is supported at this time_).
 * An Intel IA-64 (x86_64), or an ARMv8 processor (aarch64).
 * Rust's [`Cargo`](https://doc.rust-lang.org/cargo/) (v1.33.0 or higher), to build the project and all its dependencies.
 * At least one Ethernet interface(s), see [`conf/rvrrpd.conf`](conf/rvrrpd.conf) for a basic TOML configuration example.
 * Root privileges, required to access raw sockets, configure interfaces and to add kernel routes.
 * The [`libnl-3`](https://www.infradead.org/~tgr/libnl/) and `libnl-route-3` libraries for accessing the netlink interface.

# Build and run
To quickly build a development version of [`rVRRPd`](https://github.com/e3prom/rVRRPd), first make sure you have the **latest** version of [`Cargo`](https://doc.rust-lang.org/cargo/) installed. The recommended steps are to first [install](https://doc.rust-lang.org/cargo/getting-started/installation.html) Cargo, then the GNU Compiler Collection (GCC) toolchain and lastly the `libnl-3` development packages (including headers files), namely `libnl-3-dev` and `libnl-route-3-dev` on Debian and derivatives.

To quickly build the daemon executable, use the `make` or `cargo build --release` command:
```console
$ cargo build --release
   [...]
   Compiling ryu v1.0.0
   Compiling itoa v0.4.4
   Compiling ctrlc v3.1.2
   Compiling serde_json v1.0.40
   Compiling rVRRPd v0.1.1 (/home/e3prom/rVRRPd)
    Finished release [optimized] target(s) in 9.63s
```

Then install the `rvrrpd` executable on your system path by entering `make install`.

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
iftype = "macvlan"
vifname = "vrrp0"
auth_type = "rfc2338-simple"
auth_secret = "thissecretnolongeris"


[protocols]
    [[protocols.static]]
    route = "0.0.0.0"
    mask = "0.0.0.0"
    nh = "10.240.0.254"
```

The above configuration do the following:
 * Starts the daemon in foreground mode with a debug level of `5` (extensive).
 * Runs one virtual-router with group id `1` on interface `ens192.900`, with the below parameters:
   * Uses the virtual IP address `10.100.100.1`.
   * Is configured with the highest priority of `254`.
   * Has preemption enabled.
   * Has compatibility with [`RFC3768`](https://tools.ietf.org/html/rfc3768) turned on (may be required to fully interoperate with some vendors).
   * Uses the network driver `libnl` which leverage the netlink protocol. Alternatively, you can use the `ioctl` driver, which is simpler but will removes the interface's IP addresse(s) for the VIP when in Master state.
   * Is configured for a `macvlan` type interface, a MAC-based virtual interface.
   * Name the child virtual interface `vrrp0`, the latter will be used to holds the virtual router IP address.
   * Set authentication to the [`RFC2338`]'s (https://tools.ietf.org/html/rfc2338) `Simple Password` authentication method.
   * Set the secret key (or password) to be shared between the virtual routers.
* When master, install a static default route with a next-hop of `10.240.0.254`.

Finally run the binary executable `main` you just built using the command-line parameter `-m1`, to start the daemon in foreground mode:
```bash
$ sudo rvrrpd -m1 -c conf/rvrrpd.conf
Starting rVRRPd
[...]
```

Your virtual router will now listen for VRRPv2 packets and will take the `Master` or `Backup` role. If the router owns the virtual IP address, it will automatically take the `Master` role with a priority of `255`.

# Support
If you are experiencing any stability, security or interoperability problems, feel free to open a [new issue](https://github.com/e3prom/rVRRPd/issues/new).
