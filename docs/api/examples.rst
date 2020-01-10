.. _api_examples:

Client API Queries Examples
===========================

Getting Virtual Router States
-----------------------------
Getting Started
^^^^^^^^^^^^^^^
You can get running information directly from an instance of **rVRRPd** using
the HTTP Client API, but first you must authenticate using an HTTP ``POST``
request to the ``auth/`` path.

Authenticating
^^^^^^^^^^^^^^
The below example shows how to authenticate to the daemon running on
``10.0.0.1``, using the ``curl`` utility:

.. code-block:: console

    $ curl -k -c /tmp/rvrrpd-api-cookie -d "user=admin passwd=banana" -X POST https://10.0.0.1:7080/auth

The above command will send an HTTP ``POST`` request to the API, and if
successful will store the resulting session cookie to
``/tmp/rvrrpd-api-cookie``.

Requesting VRRP Information
^^^^^^^^^^^^^^^^^^^^^^^^^^^
Once authenticated, you can query the router for the current VRRP running
information by sending an HTTP ``GET`` request to the ``run/vrrp`` resource
path:

.. code-block:: console

    $ curl -k -s -b /tmp/rvrrpd-api-cookie -X GET https://10.0.0.1:7080/run/vrrp | jq

You should get a JSON formatted response like below:

.. code-block:: json

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

