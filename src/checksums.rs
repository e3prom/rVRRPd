//! checksums related functions module
//! This module is dedicated to internet checksums functions.
//!
//! credit to ref. impl. https://github.com/m-labs/smoltcp/blob/master/src/wire/ip.rs
use byteorder::{ByteOrder, NetworkEndian};

const RFC1071_CHUNK_SIZE: usize = 32;

// rfc1071() function
/// compute rfc1071 internet checksum
/// returns all-ones if carried checksum is valid
pub fn rfc1071(mut data: &[u8]) -> u16 {
    let mut acc = 0;

    // for each 32 bytes chunk
    while data.len() >= RFC1071_CHUNK_SIZE {
        let mut d = &data[..RFC1071_CHUNK_SIZE];
        while d.len() >= 2 {
            // sum adjacent pairs converted to 16 bits integer
            acc += NetworkEndian::read_u16(d) as u32;
            // take the next 2 bytes for the next iteration
            d = &d[2..];
        }
        data = &data[RFC1071_CHUNK_SIZE..];
    }

    // if it does not fit a 32 bytes chunk
    while data.len() >= 2 {
        acc += NetworkEndian::read_u16(data) as u32;
        data = &data[2..];
    }

    // add odd byte is present
    if let Some(&v) = data.first() {
        acc += (v as u32) << 8;
    }

    propagate_carries(acc)
}

// propagate final complement?
pub fn propagate_carries(word: u32) -> u16 {
    let sum = (word >> 16) + (word & 0xffff);
    ((sum >> 16) as u16) + (sum as u16)
}

// one_complement_sum() function
/// returns all-zeros if checksum is valid
pub fn one_complement_sum(data: &[u8], pos: Option<usize>) -> u16 {
    let mut sum = 0u32;
    let mut idx = 0;

    while idx < data.len() {
        match pos {
            // if a position is given:
            Some(p) => {
                if idx == p {
                    idx = p + 2; // skip 2 bytes
                }
                // if we reach the end of slice, we are done
                if idx == data.len() {
                    break;
                }
            }
            None => (),
        }
        let word = (data[idx] as u32) << 8 | data[idx + 1] as u32;
        sum = sum + word;
        idx = idx + 2;
    }

    while sum >> 16 != 0 {
        sum = (sum >> 16) + (sum & 0xFFFF);
    }

    !sum as u16
}
