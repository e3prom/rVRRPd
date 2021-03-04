.. _installation:

Install rVRRPd
==============

**rVRRPd** can be installed from source or by using pre-compiled binaries
packages. The latter is recommended for production uses, as the executables
have been previously tested for stability.

.. _requirements:

Software Requirements
---------------------
  * The Linux or `FreeBSD <https://www.freebsd.org>`_ operating system (64 bits)
  * The `OpenSSL <https://www.openssl.org/>`_ library
  * The `Netlink Protocol Library Suite <https://www.infradead.org/~tgr/libnl/>`_ library *(Linux)*

Hardware Requirements
---------------------
  * An Intel IA-64 (x86_64) or ARMv8 (aarch64) processor
  * At least **one** Ethernet interface

Source Installation
-------------------
Getting Started
^^^^^^^^^^^^^^^
To install **rVRRPd** from source, first of all, make sure you have all the
required build dependencies (see :ref:`dependencies` section below).

Then download the source tarball files (tar.gz) from our `release <https://github.com/e3prom/rVRRPd/releases>`_ page
or use `git <https://git-scm.com/>`_ to clone the source repository.

Below we will describe the step-by-step instructions on how to install
a stable release of the daemon and its utilities:

.. _dependencies:

Building Dependencies
^^^^^^^^^^^^^^^^^^^^^
To build **rVRRPd** from source you must have several programs and libraries installed on your system (preferably system-wide):
 * Rust `Cargo <https://doc.rust-lang.org/cargo/getting-started/installation.html>`_ (v1.33.0 or later), \
   to build the project and its related dependencies (crates).
 * The `OpenSSL <https://www.openssl.org/>`_ development headers
 * The `Netlink Protocol Library Suite <https://www.infradead.org/~tgr/libnl/>`_ development headers *(Linux)*

On `Debian <https://www.debian.org>`_ and derivatives, all three libraries' headers files can be installed with the below command:

.. code-block:: console

    $ sudo apt-get install libnl-3-dev libnl-route-3-dev libssl-dev

Cloning Source Repository
^^^^^^^^^^^^^^^^^^^^^^^^^
We will now clone the source from our official `github repository <https://github.com/e3prom/rVRRPd>`_:

.. code-block:: console

    $ git clone https://github.com/e3prom/rvrrpd
    Cloning into 'rvrrpd'...
    remote: Enumerating objects: 16, done.
    remote: Counting objects: 100% (16/16), done.
    remote: Compressing objects: 100% (12/12), done.
    remote: Total 1301 (delta 4), reused 12 (delta 4), pack-reused 1285
    Receiving objects: 100% (1301/1301), 347.88 KiB | 0 bytes/s, done.
    Resolving deltas: 100% (831/831), done.

Switching to Stable Release
^^^^^^^^^^^^^^^^^^^^^^^^^^^
We move to the ``rvrrpd`` directory just created by git and we will
switch to the latest stable release (here version ``0.1.3``):

.. code-block:: console

    $ cd rvrrpd
    $ git checkout tags/0.1.3
    [...]

Invoking the Build Process
^^^^^^^^^^^^^^^^^^^^^^^^^^
Enter the ``make`` command to start the build process.
Rust Cargo will automatically fetch and build all the required dependencies and
will start the build process of the **rVRRPd** daemon and related utilities
such as ``rvrrpd-pw``:

.. code-block:: console

    $ make
    Updating crates.io index
    [...]
    Compiling rVRRPd v0.1.3 (/var/tmp/rvrrpd)
        Finished release [optimized] target(s) in 2m 40s


Once the build process is completed, you can find the daemon executable
in ``target/release/rvrrpd``. The latter can be executed as-is or can be
installed system-wide (recommended).

Installing System-wide
^^^^^^^^^^^^^^^^^^^^^^
We will now install ``rvrrpd``, its accompanying configuration file
``/etc/rvrrpd.conf``, and the ``rvrrpd-pw`` utility in our system
paths by using the ``make install`` command (requires root privileges):

.. code-block:: console

    $ sudo make install
    cd utils/rvrrpd-pw && make install
    make[1]: Entering directory 'utils/rvrrpd-pw'
    [ ! -d /usr/bin ] && mkdir -p /usr/bin
    cp target/release/rvrrpd-pw /usr/bin/rvrrpd-pw
    chmod 755 /usr/bin/rvrrpd-pw
    make[1]: Leaving directory 'utils/rvrrpd-pw'
    [ ! -d /usr/sbin ] && mkdir -p /usr/sbin
    cp target/release/rvrrpd /usr/sbin/rvrrpd
    chmod 755 /usr/sbin/rvrrpd
    [ ! -d /etc/rvrrpd ] && mkdir -p /etc/rvrrpd

Configuring
^^^^^^^^^^^
Prior to running the daemon, you must edit the main configuration file
according to your network or high-availability environment. See
:ref:`Configure <config_example>` below for a basic sample configuration
example.

Running
^^^^^^^
**rVRRPd** supports multiple operating modes: it can run in ``foreground``
mode from a terminal or in ``background`` mode as a standard Unix daemon,
using the ``-m1`` and ``-m2`` switches, respectively.

.. warning::

  The daemon requires root privileges to run successfully. The daemon must
  have access to raw sockets, and to privileged kernel functions to create
  virtual interfaces, IP addresses and routes.

In the below example, we are running the daemon in ``foreground`` mode
using the ``-m1`` switch:

.. code-block:: console

    $ sudo rvrrpd -m1


Binary Package Installation
---------------------------
**rVRRPd** could also be installed directly from binaries packages.
This is the recommended way of installing the VRRP daemon for production uses
as we are testing every executable for stability prior to shipping the
releases to the public.

Getting Binary Archives
^^^^^^^^^^^^^^^^^^^^^^^
Visit the official `release <https://github.com/e3prom/rVRRPd/releases>`_ page on github and download
the latest package in ``tar.xz`` format.

You can download directly from the command-line using the ``wget`` utility:

.. code-block:: console

    $ wget "https://github.com/e3prom/rVRRPd/releases/download/0.1.3/rvrrpd-0.1.3-linux-amd64.tar.xz"

Verifying the Archives Integrity
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Prior to unpacking the archive, we strongly suggest to verify the file checksum
to ensure it has not be tempered by a third party.

.. code-block:: console

  $ wget "https://github.com/e3prom/rVRRPd/releases/download/0.1.3/SHA256SUMS"
  $ sha256sum --check SHA256SUMS
  rvrrpd-0.1.3-linux-amd64.tar.xz: OK

Unpacking Archives
^^^^^^^^^^^^^^^^^^
Untar the downloaded archive using ``tar``:

.. code-block:: console

    $ tar -xvf rvrrpd-0.1.3-linux-amd64.tar.xz
    rvrrpd-0.1.3-linux-amd64/
    rvrrpd-0.1.3-linux-amd64/README.md
    rvrrpd-0.1.3-linux-amd64/conf/
    rvrrpd-0.1.3-linux-amd64/conf/rvrrpd.conf
    rvrrpd-0.1.3-linux-amd64/conf/rvrrpd.json.conf
    rvrrpd-0.1.3-linux-amd64/rvrrpd
    rvrrpd-0.1.3-linux-amd64/LICENSE

Configuring
^^^^^^^^^^^^
Move into the release ``rvrrpd-<version>-<os>-<arch>/`` directory just
created above:

.. code-block:: console

    $ cd rvrrpd-0.1.3-linux-amd64/

:ref:`Edit <config_example>` the sample configuration file in
``etc/rvrrpd.conf`` and run the daemon from the current directory:

Running
^^^^^^^
.. warning::

  The daemon requires root privileges to run successfully. The daemon must
  have access to raw sockets, and to privileged kernel functions to create
  virtual interfaces, IP addresses and routes.

.. code-block:: console

    $ sudo ./rvrrpd -m1 -c conf/rvrrpd.conf

See our configuration reference for more information about the available
configuration options.


.. _config_example:

Basic Configuration Example
---------------------------
rVRRPd read its configuration file from the default ``/etc/rvrrpd.conf``.
The later, must be configured to match your current network and
high-availability configuration. You can also overwrite the config
file path using the ``-c`` or ``--conf`` command-line switches.

Below a sample TOML configuration file of a basic VRRP first-hop router:

.. code-block:: toml
  :caption: rvrrpd.conf
  :linenos:

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

The above configuration do the following:
 * Starts the daemon in foreground mode with a debug level of ``5``\
   (extensive).
 * Enable the Client API with the ``http`` listener \
   (listen by default on ``tcp/7080``).
 * Runs one virtual-router with group id ``1`` on interface ``ens192.900``, \
   with the below parameters:

   * Uses the virtual IP address ``10.100.100.1``.
   * Is configured with the highest priority of ``254``.
   * Has preemption enabled.
   * Has compatibility with `RFC3768 <https://tools.ietf.org/html/rfc3768>`_ turned on \
     (may be required to fully interoperate with some equipment vendors).
   * Uses the network driver ``libnl`` which leverage the netlink protocol. \
     Alternatively, you can use the ``ioctl`` driver, which is simpler but
     will removes the interface's IP addresse(s) for the VIP when in Master \
     state.
   * Is configured for a ``macvlan`` type interface, \
     a MAC-based virtual interface.
   * Name the child virtual interface ``vrrp0``, the latter will be used to \
     hold the virtual router IP address.
   * Set authentication to the `RFC2338 <https://tools.ietf.org/html/rfc2338>`_ \
     , ``Simple Password`` authentication method.
   * Set the secret key (or password) to be shared between the virtual routers.
 * When Master, install a static default route with a next-hop of \
   ``10.240.0.254``.
 * The Client API only authorizes queries from the users listed in the \
   ``users`` list under the ``[api]`` section. The users must \
   authenticate prior to  accessing the virtual router's information.

   * You can generate users passwords hashes using the \
     `rvrrpd-pw <https://github.com/e3prom/rVRRPd/tree/master/utils/rvrrpd-pw>`_ utility.

You can consult our configuration guide to have more details and
explanation about all the available configuration options.
