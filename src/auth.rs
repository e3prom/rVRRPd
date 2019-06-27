//! authentication module
use super::*;

// hmac
extern crate hmac;
use hmac::{Hmac, Mac};

// sha2
extern crate sha2;
use sha2::Sha256;

// type aliases
type HmacSha256 = Hmac<Sha256>;

// gen_auth_data() function
pub fn gen_auth_data(autht: u8, secret: &Option<String>, msg: Option<&[u8]>) -> Vec<u8> {
    match autht {
        // AUTH_TYPE_SIMPLE (RFC2338 Type-1 Plain)
        AUTH_TYPE_SIMPLE => match secret {
            Some(s) => {
                let data = format!("{:\0<8}", s);
                return data.into_bytes();
            }
            None => {
                let mut data: Vec<u8> = Vec::new();
                for _ in 0..8 {
                    data.push(0);
                }
                return data;
            }
        },
        // AUTH_TYPE_P0 (PROPRIETARY-TRUNCATED-8B-SHA256)
        // This is an internal, proprietary HMAC with a fixed output of only 8
        // bytes, which should be resistant enough to keep most attacks aways.
        AUTH_TYPE_P0 => {
            let mut data: Vec<u8> = Vec::new();
            // secret key
            let key = match secret {
                Some(s) => s,
                None => "default",
            };
            // create HMAC SHA256 instance
            let mut mac = HmacSha256::new_varkey(key.as_bytes()).expect("invalid key size");
            // input message (if any)
            match msg {
                Some(m) => mac.input(m),
                None => panic!("cannot perform authentication without message"),
            }
            // get computation result
            for b in mac.result().code().iter() {
                data.push(*b);
            }
            // truncat data to 8 bytes
            data.truncate(8);
            // return data
            return data;
        }
        // no authentication
        _ => {
            let mut data: Vec<u8> = Vec::new();
            for _ in 0..8 {
                data.push(0);
            }
            return data;
        }
    }
}
