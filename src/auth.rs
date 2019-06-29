//! authentication module
use super::*;

// hmac
extern crate hmac;
use hmac::digest::{ExtendableOutput, XofReader};
use hmac::{digest::Input, Hmac, Mac};

// sha2
extern crate sha2;
use sha2::Sha256;

// sha3
extern crate sha3;
use sha3::Shake256;

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
        // This is a proprietary type for a HMAC producing fixed outputs of only 8
        // bytes, which should be resistant enough to thwart most protocol attacks.
        AUTH_TYPE_P0 => {
            let mut data: Vec<u8> = Vec::new();
            // hmac secret key
            let key = match secret {
                Some(s) => s,
                None => "",
            };
            // create HMAC-SHA256 instance
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
        // AUTH_TYPE_P1 (PROPRIETARY-XOF-8B-SHAKE256)
        // This is an internal, proprietary type using the SHAKE256 XOF
        AUTH_TYPE_P1 => {
            // secret key
            let key = match secret {
                Some(s) => s.as_bytes(),
                None => "".as_bytes(),
            };
            // create SHAKE256 instance
            let mut hasher = Shake256::default();
            // feed the hasher
            match msg {
                Some(m) => {
                    let km = [key, m].concat();
                    hasher.input(km);
                }
                None => panic!("cannot perform authentication without message"),
            }
            // read result
            let mut reader = hasher.xof_result();
            let mut data = [0u8; 8];
            reader.read(&mut data);
            return data.to_vec();
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
