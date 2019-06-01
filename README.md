[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/e3prom/rVRRPd/blob/master/LICENSE)
[![stability-unstable](https://img.shields.io/badge/stability-unstable-yellow.svg)]()
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/e3prom/rVRRPd/graphs/contributors)

# Introduction
[rVRRPd](https://github.com/e3prom/rVRRPd) is an open and secure VRRPv2 implementation written in Rust, a rather new programming language known for its inherent memory-safety and speed.

# Development
This project is still in early development stage and only support the Linux operating-system. There is no stable API, configuration or even documentation at this time. Please keep in mind that the daemon may not be fully interoperable with standard-compliant network equipments and may exhibit stability or even security issues.

# Features
 * Aimed to be Fast and _Secure_ (see development above)
 * Multithreaded operation (1 thread per VRRP group/interface pair)
 * Easily configurable using [TOML](https://github.com/toml-lang/toml)
 * Interoperable with VRRPv2 ([RFC3768](https://tools.ietf.org/html/rfc3768)) devices
 
 # Support
If you are experiencing any stability, security or interoperability problems, feel free to open a [new issue](https://github.com/e3prom/rVRRPd/issues/new).
