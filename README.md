[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/e3prom/rVRRPd/blob/master/LICENSE)
![GitHub top language](https://img.shields.io/github/languages/top/e3prom/rvrrpd.svg)
![GitHub issues](https://img.shields.io/github/issues/e3prom/rvrrpd.svg)
![GitHub last commit](https://img.shields.io/github/last-commit/e3prom/rvrrpd.svg)
![Github build status](https://github.com/e3prom/rVRRPd/workflows/Build/badge.svg)
[![Documentation Status](https://readthedocs.org/projects/rvrrpd/badge/?version=latest)](https://rvrrpd.readthedocs.io/en/latest/?badge=latest)
[![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=TWE8MESRMWRG8)

# Introduction
[rVRRPd](https://github.com/e3prom/rVRRPd) is a fast, multi-platform and standard-compliant VRRPv2 implementation written in [Rust](https://www.rust-lang.org/), a modern programming language known for its portability, memory-safety and speed.

# Features
 * Aimed to be Fast, Portable and **Highly Secure**
 * Supports multiple operating systems and processor architectures
 * Multi-threaded operation (1 thread per interface and virtual router)
 * Easily configurable using [TOML](https://github.com/toml-lang/toml) or [JSON](https://www.json.org/)
 * Interoperable with [`RFC3768`](https://tools.ietf.org/html/rfc3768) (VRRPv2) compliant devices
   * Fully compatible with Cisco IOS and Cisco IOS-XE devices
 * Authentication Support
   * Password Authentication (Type-1) based on [`RFC2338`](https://tools.ietf.org/html/rfc2338)
   * Proprietary P0 HMAC (SHA256 truncated to 8 bytes)
   * Proprietary P1 (SHAKE256 XOF)
 * Supports multiple operation modes:
   * Sniffer mode (`-m0`)
   * Virtual Router in foreground mode (`-m1`)
   * Virtual Router in daemon mode (`-m2`)
 * Supports MAC-based Virtual LAN interface (`macvlan`) _(Linux)_
 * Uses Berkeley Packet Filters Sockets (`BPF`) _(FreeBSD)_
 * Supports BPF Linux Socket Filters (_Linux_)
 * Provides a Client Application Programming Interface (API)
   * Runs plain-text HTTP or HTTPS (SSL/TLS)

# Development
This project is still in **_active development_**, and at this time, only supports the Linux and FreeBSD operating systems. There is no stable API, configuration or even documentation yet. [`rVRRPd`](https://github.com/e3prom/rVRRPd) may not be interoperable with standard-compliant network equipments when using proprietary features (such as P0 or P1 authentication).

The development roadmap for the upcoming `0.2.0` release can be found [here](https://github.com/e3prom/rVRRPd/projects/2).

# Documentation
You can access the latest documentation at [rvrrpd.rtfd.io](https://rvrrpd.rtfd.io).

# Support
If you are experiencing any stability, security or interoperability isues, feel free to open a [new issue](https://github.com/e3prom/rVRRPd/issues/new).
