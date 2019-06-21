//! linux specific drivers module
#[allow(non_camel_case_types)]
// network drivers enumerator
#[derive(Debug)]
pub enum NetDrivers {
    ioctl, // ioctl
    libnl, // netlink (libnl-3)
}
