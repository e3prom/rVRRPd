//! protocols module
//! This module includes networking protocols data structures and related functions.

/// Protocols Structure
#[derive(Debug)]
pub struct Protocols {
    pub r#static: Option<Vec<Static>>,
}

// Protocols Type Imlementation
impl Protocols {
    // new() method
    pub fn _new(r#static: Option<Vec<Static>>) -> Protocols {
        Protocols { r#static }
    }
}

/// Static Protocol Structure
#[derive(Debug)]
pub struct Static {
    route: [u8; 4],
    mask: [u8; 4],
    nh: [u8; 4],
    metric: i16,
    mtu: u64,
}

// Static Protocol Type Implementation
impl Static {
    // new() method
    pub fn new(route: [u8; 4], mask: [u8; 4], nh: [u8; 4], metric: i16, mtu: u64) -> Static {
        Static {
            route,
            mask,
            nh,
            metric,
            mtu,
        }
    }
    // route() getter
    pub fn route(&self) -> [u8; 4] {
        self.route
    }
    // mask() getter
    pub fn mask(&self) -> [u8; 4] {
        self.mask
    }
    // nh() getter
    pub fn nh(&self) -> [u8; 4] {
        self.nh
    }
    // metric() getter
    pub fn metric(&self) -> i16 {
        self.metric
    }
    // mtu() getter
    pub fn mtu(&self) -> u64 {
        self.mtu
    }
}
