[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/e3prom/rVRRPd/blob/master/LICENSE)
![GitHub top language](https://img.shields.io/github/languages/top/e3prom/rvrrpd.svg)
![GitHub issues](https://img.shields.io/github/issues/e3prom/rvrrpd.svg)
![GitHub last commit](https://img.shields.io/github/last-commit/e3prom/rvrrpd.svg)
![Github build status](https://github.com/e3prom/rVRRPd/workflows/Build/badge.svg)
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

# Dependencies
 * A Linux or FreeBSD 64-bits operating system.
 * An Intel IA-64 (x86_64), or an ARMv8 processor (aarch64).
 * Rust's [`Cargo`](https://doc.rust-lang.org/cargo/) (v1.33.0 or higher), to build the project and all its dependencies.
 * At least one Ethernet interface, see [`conf/rvrrpd.conf`](conf/rvrrpd.conf) for a basic configuration example.
 * Root privileges, required to access raw sockets, configure interfaces and to add routes.
 * The [`libnl-3`](https://www.infradead.org/~tgr/libnl/) and `libnl-route-3` libraries for accessing the netlink interface (Linux).
 * The `libssl-dev` package for OpenSSL development headers files.

# Build from sources
To quickly build a development version of [`rVRRPd`](https://github.com/e3prom/rVRRPd), first make sure you have the **latest** version of [`Cargo`](https://doc.rust-lang.org/cargo/) installed. The recommended steps are to first [install](https://doc.rust-lang.org/cargo/getting-started/installation.html) Cargo, then the GNU Compiler Collection (GCC) toolchain and lastly the `libnl-3` development packages (including headers files), namely `libnl-3-dev` and `libnl-route-3-dev`, on Linux Debian and derivatives.

To quickly build the daemon executable, use the `make` or `cargo build --release` command:
```console
$ cargo build --release
   [...]
   Compiling tokio v0.1.21
   Compiling foreign-types-macros v0.1.0
   Compiling serde_derive v1.0.92
   Compiling foreign-types v0.4.0
   Compiling rVRRPd v0.1.3 (/home/e3prom/rVRRPd)
    Finished release [optimized] target(s) in 9.62s
```

Then install the `rvrrpd` executable on your system by entering the `make install` command.

# Binaries
You can also run `rvrrpd` using pre-compiled binaries available in the [release](https://github.com/e3prom/rVRRPd/releases) page. These binaries are stable enough to be used in production, however if you need the latest features, please use the sources from the Master branch instead.

# Running
### Configuring
Before running the VRRP daemon, copy the example configuration file at [`conf/rvrrpd.conf`](conf/rvrrpd.conf) to the default configuration file path `/etc/rvrrpd/rvrrpd.conf`. Then use your favorite text editor to configure the virtual router(s) to your needs.

See below for an example of a virtual router running on an Ethernet interface with password authentication and preemption enabled:
```TOML
debug = 5
pid = "/var/tmp/rvrrpd.pid"
working_dir = "/var/tmp"
main_log = "/var/tmp/rvrrpd.log"
error_log = "/var/tmp/rvrrpd-error.log"
client_api = "http"

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

[api]
    tls = false
    host = "0.0.0.0:7080"
    users = [ "{{SHA256}}admin:0:1eb7ac761a1201f9:095820af..." ]
```

The above configuration do the following:
 * Starts the daemon in foreground mode with a debug level of `5` (extensive).
 * Enable the Client API with the `http` listener (the later listen by default on tcp/7080).
 * Runs one virtual-router with group id `1` on interface `ens192.900`, with the below parameters:
   * Uses the virtual IP address `10.100.100.1`.
   * Is configured with the highest priority of `254`.
   * Has preemption enabled.
   * Has compatibility with [`RFC3768`](https://tools.ietf.org/html/rfc3768) turned on (may be required to fully interoperate with some equipment vendors).
   * Uses the network driver `libnl` which leverage the netlink protocol. Alternatively, you can use the `ioctl` driver, which is simpler but will removes the interface's IP addresse(s) for the VIP when in Master state.
   * Is configured for a `macvlan` type interface, a MAC-based virtual interface.
   * Name the child virtual interface `vrrp0`, the latter will be used to hold the virtual router IP address.
   * Set authentication to the [`RFC2338`]'s (https://tools.ietf.org/html/rfc2338) `Simple Password` authentication method.
   * Set the secret key (or password) to be shared between the virtual routers.
* When master, install a static default route with a next-hop of `10.240.0.254`.
* The Client API only authorizes queries from the users listed in the `users` list under the `[api]` section. The users must authenticate prior to accessing the virtual router's information.
  * You can generate users passwords hashes using the [`rvrrpd-pw`](https://github.com/e3prom/rVRRPd/tree/client-api/utils/rvrrpd-pw) utility.

### Starting the daemon

Finally run the executable using the command-line parameter `-m1`, to start the daemon in foreground mode:
```bash
$ sudo rvrrpd -m1 -c conf/rvrrpd.conf
Starting rVRRPd
[...]
```

Your virtual router will now listen for VRRP packets and will take the `Master` or `Backup` role. If the router owns the virtual IP address, it will automatically take the `Master` role with a priority of `255`.

### Querying the Client API
You can get running information directly from the VRRP router using the HTTP Client API, but first you must authenticate using a `POST` request to the `auth/` path. The below example shows how to authenticate to the daemon running on `10.0.0.1`, using curl:
```bash
$ curl -k -c /tmp/rvrrpd-api-cookie -d "user=admin passwd=banana" -X POST https://10.0.0.1:7080/auth
```

The above command will send a POST request to the API, and if successful will store the resulting session cookie to `/tmp/rvrrpd-api-cookie`. Once authenticated, you can query the router for the current VRRP running information by sending a `GET` request to `run/vrrp`:
```bash
$ curl -k -s -b /tmp/rvrrpd-api-cookie -X GET https://10.0.0.1:7080/run/vrrp | jq
```

You should get a JSON formatted answer like below:
```json
[
  {
    "virtual_ip": "10.100.100.1",
    "group": 1,
    "interface": "standby1",
    "priority": 254,
    "preempt": true,
    "state": "Master"
  },
  {
    "virtual_ip": "10.100.101.1",
    "group": 2,
    "interface": "standby2",
    "priority": 254,
    "preempt": true,
    "state": "Master"
  }
]
```

# Donation
Help us by donating to the project. Every penny will directly cover the development costs of `rVRRPd`, which range from coffee to the bare-metal servers powering the interporability and testing labs.

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=TWE8MESRMWRG8)

You can donate by Paypal using the above button, or by using a crypto currency listed below:

| Crypto Currency     | Wallet Address                                           | Memo ID    |
| ------------------- | -------------------------------------------------------- | ---------- |
| Bitcoin (BTC)       | 3Pz7PQk5crAABg2MsR6PVfUxGzq2MmPd2i                       |            |
| Etherum (ETH)       | 0x0686Dd4474dAA1181Fc3391035d22C8e0D2dA058               |            |
| Stellar Lumen (XLM) | GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37 | 3006351358 |


# Support
If you are experiencing any stability, security or interoperability isues, feel free to open a [new issue](https://github.com/e3prom/rVRRPd/issues/new).
