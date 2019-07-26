//! generic drivers module

// network drivers enumerator
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum NetDrivers {
    ioctl, // ioctl
    libnl, // netlink (libnl-3)
}

// network interfaces type enumerator
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum IfTypes {
    ether,   // default ethernet
    macvlan, // macvlan
}

// pflag operation Enumerator
pub enum PflagOp {
    Set,
    Unset,
}

// Operation enumerator
#[derive(Debug)]
pub enum Operation {
    Add, // Add IP address
    Rem, // Remove IP Address
}
