//! rVRRPd-pw - rVRRPd password utility
//! This program provides a fast and easy way to generate hashed passwords for rVRRPd,
//! a fast and highly-secure VRRP daemon.

// std
use std::error::Error;
use std::fmt;

// clap
use clap::{App, Arg, crate_version};

// rand
use rand::prelude::Rng;

// sha256
use sha2::{Digest, Sha256};

// scrypt
use scrypt::{scrypt_simple, ScryptParams};

// MyError type
#[derive(Debug)]
struct MyError {
    msg: String,
}

impl MyError {
    fn new(msg: &str) -> MyError {
        MyError {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        &self.msg
    }
}

// main() function
fn main() {
    let matches = App::new("rvrrpd-pw")
        .version(crate_version!())
        .long_version(concat!(crate_version!(), "
Copyright (C) 2019-2021 Nicolas Chabbey.
License GPLv3+: GNU GPL Version 3 or any later version <https://www.gnu.org/licenses/gpl-3.0.txt>.
This program comes with ABSOLUTELY NO WARRANTY. This is free software,
and you are welcome to redistribute it under certain conditions.

Written by Nicolas Chabbey <eprom@toor.si>."))
        .author("Nicolas Chabbey <eprom@toor.si>")
        .about("Quick and easy password generation for rVRRPd")
        .arg(
            Arg::with_name("user")
                .short("u")
                .long("user")
                .takes_value(true)
                .help("user name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .takes_value(true)
                .help("user's password")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("alg")
                .short("a")
                .long("hash-alg")
                .takes_value(true)
                .help("hashing algorithm (default: sha256)")
                .index(3),
        )
        .after_help(
            "HASHING ALGS:\n\
        sha256\t\tSHA2 (256 bits)\n\
        scrypt\t\tscrypt (interactive)\n",
        )
        .get_matches();

    let user = matches.value_of("user").unwrap();
    let passwd = matches.value_of("password").unwrap();
    let alg = matches.value_of("alg").unwrap_or("sha256");

    match gen_hashed_pw(user, passwd, alg) {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        Ok(_) => std::process::exit(0),
    }
}

/// gen_hashed_pw() function
/// Print new user account information
fn gen_hashed_pw(user: &str, passwd: &str, alg: &str) -> Result<(), MyError> {
    match alg {
        "sha256" => {
            let mut rng = rand::thread_rng();
            let r = rng.gen::<u64>();
            let salt = format!("{:x}", r);
            match gen_sha256_hash(&passwd, &salt) {
                Some(h) => {
                    display_userpw_line(alg, user, Some(salt), h);
                }
                None => {
                    let err = format!("the {} hashing function failed", alg);
                    return Err(MyError::new(&err));
                }
            }
        }
        "scrypt" => match gen_scrypt_hash(passwd) {
            Some(h) => {
                display_userpw_line(alg, user, None, h);
            }
            None => {
                let err = format!("the {} hashing function failed", alg);
                return Err(MyError::new(&err));
            }
        },
        _ => {
            let err = format!("unknown hashing algorithm {}", alg);
            return Err(MyError::new(&err));
        }
    }

    Ok(())
}

/// gen_sha256_hash() function
/// Generate a SHA256 hashed password with random salt
fn gen_sha256_hash(passwd: &str, salt: &str) -> Option<String> {
    // create new sha256 hasher
    let mut hasher = Sha256::new();
    // feed the hasher with the user supplied password
    hasher.update(&passwd);
    // feed the hasher with the supplied salt
    hasher.update(salt);
    // get the result
    let h = hasher.finalize();
    // format the result in a hexadecimal string
    let hash = format!("{:x}", h);
    // return the hash
    Some(hash)
}

/// gen_scrypt_hash() function
/// Generate a scrypt based password
fn gen_scrypt_hash(passwd: &str) -> Option<String> {
    // create scrypt parameters with sound values
    let params = ScryptParams::new(4, 8, 1).unwrap();
    // hash the given password
    let hash = scrypt_simple(&passwd, &params).expect("PNRG failure");
    // return the hash
    Some(hash)
}

/// display_userpw_line() function
/// Display the user password line for inclusion in rVRRPd configuration
fn display_userpw_line(alg: &str, user: &str, salt: Option<String>, hash: String) {
    // not implemented yet
    let level = 0;
    println!(
        "{{{{{}}}}}{}:{}:{}:{}",
        alg.to_uppercase(),
        user,
        level,
        salt.unwrap_or_else(|| "".to_string()),
        hash
    );
}
