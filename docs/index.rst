.. rVRRPd documentation master file, created by
   sphinx-quickstart on Fri Jan  3 13:40:25 2020.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

Welcome to rVRRPd's documentation!
==================================
**rVRRPd** is a fast, secure and standard compliant implementation
of the high-availability VRRP protocol. It is very scalable, and can
run on multiple platforms and operating systems.

As its name implies, **rVRRPd** can run as a Unix daemon or
as a standalone program. It can also exposes a RESTful API for
monitoring and configuration purposes, enabling Software Defined
Networking (SDN) applications.

Features
--------
rVRRPd supports a number of innovative features, some of which allow for a more efficient and secure operation of the protocol:
 - Secure software architecture leveraging the `Rust <https://www.rust-lang.org/>`_ programming language
 - Highly scalable; up to several hundreads of concurrent VRRP groups
 - Supports standard `RFC3768 <https://tools.ietf.org/html/rfc3768>`_ and \
   `RFC2338 <https://tools.ietf.org/html/rfc2338>`_, ``Simple Password`` authentication
 - Supports additional :ref:`proprietary authentication
   <config/reference:auth_type>` methods
 - Supports multiple operating systems and processors architectures
 - Provides a RESTful Client Application Programming Interface (API)
 - Provides a plaintext HTTP or SSL/TLS HTTPS interface to the Client API
 - Leverages additional features such as ``macvlan`` and \
   ``Linux Socket Filters``

Features Support Matrix
^^^^^^^^^^^^^^^^^^^^^^^
+-----------------------------------------------+-------+---------+
| Supported Features                            | Linux | FreeBSD |
+===============================================+=======+=========+
| Multiple Listeners Threads                    | Yes   | Yes     |
+-----------------------------------------------+-------+---------+
| RESTful Client API                            | Yes   | Yes     |
+-----------------------------------------------+-------+---------+
| Socket Filters (eBPF)                         | Yes   | No      |
+-----------------------------------------------+-------+---------+
| MAC-Based Virtual LAN Interface (``macvlan``) | Yes   | No      |
+-----------------------------------------------+-------+---------+
| Static Routing                                | Yes   | No      |
+-----------------------------------------------+-------+---------+

Configuration Guide
-------------------
This part of the documentation focuses on the step-by-step installation
instructions of the daemon and on how to configure the latter for various
network and high-availability scenarios.

.. toctree::
   :maxdepth: 2

   config/intro
   config/install
   config/reference

Client API Guide
----------------
This guide covers the Client Application Programming Interface (API), how to
configure it, how to make requests and interprets their various
responses.

.. toctree::
   :maxdepth: 2

   api/intro
   api/config
   api/examples

Additional resources
--------------------
* `Github Repository <https://github.com/e3prom/rVRRPd>`_
* :ref:`genindex`
* :ref:`search`
