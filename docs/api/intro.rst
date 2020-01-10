.. _api-introduction:

Introduction to the API
=======================
**rVRRPd** provides an Application Programming Interface (API) that allow
remote tasks to be performed on the daemon and on the running virtual
routers.

The Client API can be accessed by various means, but at this time of
writing, only supports the Hypertext Transfer Protocol (HTTP), in
plain-text or securely by using a SSL/TLS channel. The use of the
latter is highly recommended for integrity and confidentiality
purposes.

RESTful HTTP Interface
----------------------
The Client API can be accessed over HTTP using the Representational
State Transfer or REST model, which provides a simple and uniform
access model to the various data coming from the daemon instance,
the VRRP virtual routers, and from the operating system such as
interfaces information and kernel routes.

The API not only allow to read data and to parse it efficiently,
but also to make modification to the running instance of **rVRRPd**,
such as adding a new virtual router, or changing its priority so it
can take over a Master router.

.. note::

    As of version 0.1.3, the Client API is only providing read-only
    access. Modifications are not yet supported but will be introduced
    in a later release.

To query **rVRRPd** for information, such as the current role of a
running VRRP virtual router, a simple HTTP GET request can be made
to a specific resource path. If the query can be honored, the API
will return a `JSON <https://en.wikipedia.org/wiki/JSON>`_ formated
body response with all the attributes and values corresponding to
your query.

The responses can be easily and efficiently parsed by both a human
and a machine, thus providing a uniform and standardize interface
that can be used as a *console*, as an automation interface for
SDN applications and much more.
