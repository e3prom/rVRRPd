.. _config-introduction:

Introduction
============
By default, **rVRRPd** reads the ``/etc/rvrrpd.conf`` configuration file.
This file holds all the configuration elements needed for the proper operation
of the daemon, the virtual routers, and their related functions.

At this time of writing, both `TOML <https://github.com/toml-lang/toml>`_ (default)
and the `JSON <https://en.wikipedia.org/wiki/JSON>`_ formats are supported for
the main configuration file. The former is usually simpler to understand and
to write, greatly reducing human errors. JSON based configurations however,
are harder to write and to parse for some people, but may be more practical
when used with automation tools or with an HTTP based Application Programming
Interface (API).

If you don't know which configuration file format to use, we recommend to
stick with `TOML <https://github.com/toml-lang/toml>`_, unless you want to
use the Client API extensively.

The **rVRRPd** daemon runs one ``virtual-router`` per ``interface, group``
pair, which means you can configure the same VRRP groups id or
``virtual-router`` id across several physical interfaces. The daemon can
scale to hundreds if not thousands of active virtual-routers if the CPU
and memory resources permit.

The initial :ref:`developer <developers>` of **rVRRPd** has chosen to build the daemon
entirely using the `Rust <https://www.rust-lang.org/>`_ programming language.
Rust is a language, aimed primarily at security and speed. You get all the
benefits of a modern object-oriented programming language such as Java or C++,
without their respective performance penalty and inherent security risks.

We tried to keep ``unsafe`` blocks as small as possible in order to provides
a clean interface to unsafe functions. However, we cannot removes all of them
as they are necessary to implement functions calls to the standard C library,
and to the various interfaces (such as IOCTLs) to the operating system kernel.

We hope that you will enjoy running **rVRRPd** and you would be able to solve
your current network and high-availability challenges in less time and thus
without the hassles commonly found in commercial solutions.

This project wouldn't be live without the dedication of its developers, and
the open source community. Any `contributions <https://github.com/e3prom/rVRRPd>`_
are greatly welcome, and will help us developing new features and a more
stable and secure implementation.

.. _developers:

Developpers
^^^^^^^^^^^
   * Nicolas Chabbey

     * Keybase: `@e3prom <https://keybase.io/e3prom>`_
     * PGP Public Key Fingerprint: \
       ``DBD4 3BD8 81F3 C3E2 37E1 9E54 D7FF 004E 2E22 CF1C``

Sponsorship
^^^^^^^^^^^
You can help us directly by donating to the project.

Every single penny will cover the development cost of **rVRRPd**, which is
comprised of a lot of coffee, and the power bill of the bare-metal servers
running the interoperability and testing labs.

You can donate by Paypal, or by using a crypto-currency listed below:

.. image:: https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif
   :target: https://www.paypal.com/cgi-bin/webscr?cmd=_s-xclick&hosted_button_id=TWE8MESRMWRG8

+---------------------+--------------------------------------------+
| Crypto Currency     | Wallet Address                             |
+=====================+============================================+
| Bitcoin (BTC)       | 3Pz7PQk5crAABg2MsR6PVfUxGzq2MmPd2i         |
+---------------------+--------------------------------------------+
| Etherum (ETH)       | 0x0686Dd4474dAA1181Fc3391035d22C8e0D2dA058 |
+---------------------+--------------------------------------------+

Software License
^^^^^^^^^^^^^^^^
 .. include:: ../../LICENSE
    :literal:
