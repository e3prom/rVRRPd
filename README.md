[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/e3prom/rVRRPd/blob/master/LICENSE)
[![stability-unstable](https://img.shields.io/badge/stability-unstable-yellow.svg)](https://github.com/e3prom/rVRRPd/releases)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/e3prom/rVRRPd/graphs/contributors)

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
 * Virtual Router mode (-m1)

# Roadmap
 * Linux Netlink Support
 * OpenBSD Support
 * Objects Tracking
 * Interoperability Testing

# Requirements
 * Linux 64-bits kernel (only Linux is supported at this time)
 * [Rust Cargo](https://doc.rust-lang.org/cargo/) (required to compile the development version)
 * One or more Ethernet interface (see [conf/rvrrpd.conf](conf/rvrrpd.conf) for a configuration example)
 * Root privileges (required to put interfaces in promiscuous mode, access raw sockets and use of ioctls)

# Build and run
To quickly build and run the development version of [rVRRPd](https://github.com/e3prom/rVRRPd), first make sure you have Rust's [Cargo](https://doc.rust-lang.org/cargo/) installed. The recommended way is to [install](https://doc.rust-lang.org/cargo/getting-started/installation.html) the latest version of Cargo along with GNU Compiler Collection (GCC) toolchain.

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
Usage: target/release/main -m0|1 [options]>

Options:
    -h, --help          display help information
    -i, --iface INTERFACE
                        ethernet interface to listen on
    -m, --mode MODE     operation mode:
                        0(sniff), 1(foreground)
    -c, --conf FILE     path to configuration file
    -d, --debug LEVEL   debugging level:
                        0(none), 1(low), 2(medium), 3(high), 5(extensive)

```
Edit the configuration file in `conf/rvrrpd.conf` to reflect your environment, then run the built binary executable `main` using the command-line parameter `-m1` (virtual-router mode):
```
$ target/release/main -m1
```

# Support
If you are experiencing any stability, security or interoperability problems, feel free to open a [new issue](https://github.com/e3prom/rVRRPd/issues/new).
