.. _config_reference:

Configuration Reference
=======================

General Directives
------------------

debug
^^^^^
    :Description: The verbose or debugging level.
    :Value type: Decimal
    :Default: 0

    The ``debug`` directive sets the debugging (or verbosity) level
    of the daemon.

    Possible values are:
        * ``0``     Information
        * ``1``     Low
        * ``2``     Medium
        * ``3``     High
        * ``5``     Extensive

time_zone
^^^^^^^^^
    :Description: The timestamps reference time zone
    :Value type: String
    :Default: local

    The ``time_zone`` directive sets the reference time zone for
    the various daemon timestamps.

    Possible values are:
        * ``local`` for Local Time (LT)
            This setting uses the locally configured time zone of the
            operating system.
        * ``utc`` for Coordinated Universal Time (UTC)
            Timestamps will be given in UTC or Zulu time.

time_format
^^^^^^^^^^^
    :Description: The timestamps time format
    :Value type: String
    :Default: disabled

    The ``time_format`` directive sets the reference time format for
    the various daemon timestamps.

    Possible values are:
        * ``disabled`` for no particular time format (use the default
          time format)
        * ``short`` for a shortened, more concise time format
        * ``rfc2822`` for the standard `RFC2822 <https://tools.ietf.org/html/rfc2822>`_, Internet Time Format

pid
^^^
    :Description: The daemon's PID file path
    :Value type: String
    :Default: /var/run/rvrrpd.pid

    The ``pid`` directive sets the full or relative path to the daemon's
    PID file.

working_dir
^^^^^^^^^^^
    :Description: The daemon's working directory
    :Value type: String
    :Default: /tmp

    The ``working_dir`` directive sets the daemon's working directory.
    The daemon's user must have read access to this directory.

main_log
^^^^^^^^
    :Description: Path to the daemon's main log file
    :Value type: String
    :Default: /var/log/rvrrpd.log

    The ``main_log`` directive sets the path to the daemon's main log file.

error_log
^^^^^^^^^
    :Description: Path to the daemon's error log file
    :Value type: String
    :Default: disabled

    The ``error_log`` directive sets the path to the daemon's error log file.
    Any errors occuring during the runtime are written to this log file.

.. _client-api:

client_api
^^^^^^^^^^
    :Description: Client API interface type
    :Value type: String
    :Default: http

    The ``client_api`` directive sets the Client API interface type.

    Possible values are:
        * ``http`` for the RESTful HTTP interface
            This value enable a plain-text HTTP or HTTPS (SSL/TLS) interface
            to the client API. It does include user authentication and
            a secure communication channel when SSL/TLS is enabled.

.. versionadded:: 0.1.3

   Directive added with Client API Support


Virtual Routers Directives
--------------------------

group
^^^^^^
    :Description: Virtual Router Group ID (VRID)
    :Value type: Integer
    :Default: *none*

    The ``group`` directive sets the VRRP group id or virtual-router id (VRID).

    Valid values are:
        * ``0-255`` The VRRP group id or virtual-router id.
          Usually matches the sub-interface unit number or
          interface's vlan id.

interface
^^^^^^^^^
    :Description: Interface to run VRRP on
    :Value type: String
    :Default: *none*

    The ``interface`` directive sets the VRRP virtual-router's interface.
    Only Ethernet interfaces are supported.

.. _if_type:

iftype
^^^^^^
    :Description: Interface type
    :Value type: String
    :Default: *none*

    The ``iftype`` directive sets the VRRP virtual-router's interface type.
    By default, the daemon will directly work with the configured running
    interface, and therefore may change its IP and/or MAC address(es).

    Valid values are:
        * ``macvlan`` Use a MAC-Based Virtual LAN interface.

.. versionadded:: 0.1.1

   Directive added with MAC-Based Virtual LAN Interface Support

vip
^^^
    :Description: Virtual IP Address
    :Value type: String
    :Default: *none*

    The ``vip`` directive sets the VRRP standby address or virtual-router
    address. Only IPv4 addresses are currently supported at this time.

priority
^^^^^^^^
    :Description: Virtual Router Priority
    :Value type: Integer
    :Default: 100

    The ``priority`` directive sets the virtual-router VRRP priority.

    Valid values are:
        * ``1-254`` The VRRP virtual router priority. Values 0 and 255
          are reserved as per `RFC3768 <https://tools.ietf.org/html/rfc3768>`_
          and cannot be configured manually.

preemption
^^^^^^^^^^
    :Description: Preemption Support
    :Value type: Boolean
    :Default: false

    The ``preemption`` directive sets if preemption is enabled. By default,
    preemption is turned off; a higher-priority virtual router cannot preempt
    an active Master.

    Valid values are:
        * ``true`` Preemption is turned on, a higher-priority Standby
          virtual router can preempt the current Master virtual router.
        * ``false`` Preemption is turned off.

.. _auth_type:

auth_type
^^^^^^^^^
    :Description: Authentication Type
    :Value type: String
    :Default: *none*

    The ``auth_type`` directive sets the VRRP group's authentication type.
    Authentication allow to authenticate VRRP messages and with some types
    allow to verify their integrity. Authentication can prevent a
    misconfigured VRRP virtual router to take over the Master, resulting
    in the blackhole or interception of the user network traffic.

    Valid values are:
        * ``rfc2338-simple`` for `RFC2338 <https://tools.ietf.org/html/rfc2338>`_
          Simple Password Authentication.
        * ``p0-t8-sha256`` for proprietary P0 Authentication. Uses a
          SHA256 HMAC of the VRRP messages. This type provides both messages
          authentication and integrity.
        * ``p1-t8-shake256`` for proprietary P1 Authentication. Uses the
          SHAKE256 Extendable-Output Function (XOF). This type provides both
          messages authentication and integrity.

auth_secret
^^^^^^^^^^^
    :Description: Authentication Secret
    :Value type: String
    :Default: *none*

    The ``auth_secret`` directive sets the VRRP group's authentication secret
    or password. Ensure all virtual routers among the configured group share
    the same secret and that the latter has been transmitted securely.

    .. warning::

        Keep in mind that the configuration file holds the secret, therefore
        only authorized users should be able to read it.

rfc3768
^^^^^^^
    :Description: RFC3768 Compatibility Warning Flag
    :Value type: Boolean
    :Default: true

    The ``rfc3768`` directive allow you to force the compatibility flag.
    The meaning of this flag may be confusing, and can be safely ignored
    most of the time. When this flag is set to ``true``, it indicates
    the virtual router may **NOT** operates entirely according to the
    applicable VRRP RFCs. In particular regarding to the authentication
    and to the length of some VRRP PDUs header fields. When this flag is
    ``true``, the virtual router may not be interoperable with
    third-party, standard-compliant devices or softwares.

    .. note::

        Enabling proprietary features such as the proprietary authentication
        types, will automatically turn this flag on.

    Valid values are:
        * ``true`` to forcibly enable non-standard operations.
        * ``false`` to forcibly disable non-standard operations whenever
          possible.

netdrv
^^^^^^
    :Description: Network Driver
    :Value type: String
    :Default: ioctl

    The ``netdrv`` directive specify which network driver to uses for the
    virtual-router. The available drivers depend on the operating system
    and slight differences do exists between them. The driver is used
    partially or entirely to; add the virtual IP addresses, create the
    virtual interface, change the interface's MAC address, or to update
    the kernel routes.

    Valid values are:
        * ``ioctl`` for using IOCTLs. This option should be supported in
          all Linux based operating systems, even with the presence of an
          old kernel.
        * ``libnl`` for using the `Netlink Protocol Library <https://www.infradead.org/~tgr/libnl/>`_
          which is an intermediate API to communicate with the Linux
          Netlink protocol. The latter is a modern and robust way
          of configuring and interrogating the kernel.

          .. note::

            We strongly suggest to keep using this driver whenever possible.
            When using ``macvlan`` interfaces, this driver is automatically
            enabled.

vifname
^^^^^^^^
    :Description: Virtual Interface Name
    :Value type: String
    :Default: standby\<*group-id*\>

    The ``vifname`` directive sets the virtual-router's virtual interface name.
    By default, the virtual interface is named using the ``standby<group-id>``
    format, where ``group-id`` correspond to the virtual-router's VRRP group
    id or VRID.

    .. note::

        This directive is only used when virtual interface support is activated.
        (e,g. by having the :ref:`iftype <if_type>` directive set to ``macvlan``).

.. versionadded:: 0.1.1

   Directive added with MAC-Based Virtual LAN Interface Support

socket_filter
^^^^^^^^^^^^^
    :Description: Socket Filter Support
    :Value type: String
    :Default: true

    The ``socket_filter`` directive allow you to enable or disable the
    use of Socket Filters. On Linux, eBPF based Socket Filters allow
    every virtual-router raw sockets to only receives VRRP traffic
    matching their interface and VRRP group, thus greatly improving
    performance.

    Valid values are:
        * ``true`` for enabling support for socket filters. Drastically
          improves the listener threads performance by allowing the
          kernel to filter out unwanted traffic not to be processed by
          the listening thread.
        * ``false`` for disabling support for socket filters.

.. versionadded:: 0.1.2

   Directive added with Linux Socket Filters Support


API Directives
--------------

users
^^^^^
    :Description: API Users
    :Value type: List of Strings
    :Default: *none*

    The ``users`` directive lists the user accounts authorized for the
    Client API. Every string in the list must adhere to strict formatting
    rules and can be easily generated using the ``rvrrpd-pw`` utility.

secret
^^^^^^
    :Description: API Secret
    :Value type: String
    :Default: 128-bits random number

    The ``secret`` directive sets the API secret. This secret is used for
    a number of cryptogrphic functions and must be kept secret.

    By default, at every start of the daemon, a random 128 bits unsigned
    integer is generated from a secure PRNG. This number is large enough
    and *SHOULD* have sufficient entropy to provides good security.

    You can overwrite this secret by specifiy your own. The secret will
    be maintained across restart of the *rVRRPd* daemon.

    .. warning::
        Improper setting of the secret string can open up vulnerabilities
        or security holes, such as authentication bypass.

    .. note::
        If setting the secret manually, please ensure your string is long
        and random enough to provides *sufficient* security. We strongly
        recommend to use a random number generator to generate it.

host
^^^^
    :Description: Listening Host
    :Value type: String
    :Default: 0.0.0.0:7080

    The ``host`` directive sets the IP address(es) and port for the
    API interface to listen on. By default it listens on all interfaces
    on port ``7080``.

    When setting the Client API Interface to ``http`` this directive will
    specify which interfaces and port the HTTP or HTTPS service will
    listen on.

tls
^^^
    :Description: Transport Layer Security (TLS) Support
    :Value type: Boolean
    :Default: false

    The ``tls`` directive allow you to enable or disable support for
    SSL/TLS. When using the ``http`` :ref:`Client API Interface <client-api>`,
    it will allow you to enable secure HTTPS communication with the
    API clients.

    Valid values are:
        * ``true`` for activating Transport Layer Security (TLS) on
          the API interface.
        * ``false`` for disabling the TLS support.

tls_key
^^^^^^^
    :Description: SSL/TLS Key File
    :Value type: String
    :Default: /etc/rvrrpd/ssl/key.pem

    The ``tls_key`` directive allow you to set the full or relative path
    to the TLS key file.

tls_cert
^^^^^^^^
    :Description: SSL/TLS Certificate File
    :Value type: String
    :Default: /etc/rvrrpd/ssl/cert.pem

    The ``tls_key`` directive allow you to set the full or relative path
    to the certificate chain file. At this time of writting, only a
    valid X.509 server's certificate is necessary.
