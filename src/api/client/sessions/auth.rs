//! Client API - session authentication module
use super::*;

// std
use std::thread;
use std::time;

// session token
use token::SessionToken;

// config
use crate::config;

// regex
extern crate regex;
use regex::Regex;

// rand
use rand::Rng;

// sha256
use sha2::{Digest, Sha256};

// scrypt
extern crate scrypt;
use scrypt::{scrypt_check, scrypt_simple, ScryptParams};

/// auth_api_client() function
pub fn auth_api_client(
    cfg: &config::CConfig,
    user: String,
    passwd: String,
) -> Option<SessionToken> {
    // authenticate the API user
    let sess = match auth_user_from_db(cfg, user, passwd) {
        Some(usr) => {
            // if authentication is succesful, create a new SessionToken
            let mut token = SessionToken::new();
            // set authenticated user
            token.set_user(usr);
            // generate the token
            match token.gen_token(cfg) {
                // if succesfully generated
                Ok(()) => {
                    // set session duration
                    let timeout = time::Duration::new(60, 0);
                    token.set_validfor(timeout.as_secs());
                    // return SessionToken
                    Some(token)
                }
                // if token generation failed
                _ => None,
            }
        }
        // if authentication failed
        None => None,
    };

    sess
}

/// auth_user_from_db() function
/// read the 'users' vector in the api configuration section
/// and for every user, compare the hashed passwords according
/// to the configured hash function.
///
/// return the user name String if sucessfully authenticated
fn auth_user_from_db(cfg: &config::CConfig, user: String, passwd: String) -> Option<String> {
    // initialize response
    let res: Option<String> = None;

    // access configuation api users
    if let Some(a) = &cfg.api {
        for acc in a.users() {
            match regex_captures_apiuser(&acc) {
                Some(c) => {
                    let alg = c.get(1).unwrap().as_str().to_string();
                    let username = c.get(2).unwrap().as_str().to_string();
                    let _access = c.get(3).unwrap().as_str().to_string();
                    let salt = c.get(4).unwrap().as_str().to_string();
                    let hash = c.get(5).unwrap().as_str().to_string();
                    // if the username matches
                    if username == user {
                        // perform password hashing
                        match &alg[..] {
                            "SHA256" => {
                                // create new SHA256 hasher
                                let mut hasher = Sha256::new();
                                // feed the hasher with the password
                                hasher.input(&passwd);
                                // feed the hasher with the salt
                                hasher.input(&salt);
                                // convert the result to an hex formatted String
                                let h2 = format!("{:x}", hasher.result());
                                // reduce likelihood of timing attacks by introducing a random delay
                                // prior to the hashes's comparison.
                                let mut rng = rand::thread_rng();
                                let rdelay = rng.gen::<u8>() as u64;
                                let time = time::Duration::from_millis(rdelay);
                                thread::sleep(time);
                                // compare hashed values
                                if hash == h2 {
                                    return Some(username);
                                }
                            }
                            "scrypt" => {
                                // // create scrypt parameters with sound values
                                // let params = ScryptParams::new(4, 8, 1).unwrap();
                                // // hash the received password
                                // let h2 = scrypt_simple(&passwd, &params).expect("PNRG failure");
                                // // print the hashed password for debugging purpose
                                // println!("DEBUG: h2 is {}", h2);
                                // check if password is matching the stored hash
                                if scrypt_check(&passwd, &hash).is_ok() {
                                    return Some(username);
                                }
                            }
                            // if alg doesn't match, continue
                            &_ => (),
                        }
                        //return Some(username);
                    }
                }
                // if regex caputre failed, proceed to next account entry
                None => (),
            }
        }
    }

    // return authentication response
    res
}

/// regex_captures_apiuser function
fn regex_captures_apiuser(acc: &String) -> Option<regex::Captures> {
    // the API user account information is formatted as follow:
    // {{<hash-alg>}}<user-name>:<access-level>:<salt><password-hash>
    // 'user-name' must be alphanumeric between 1 and 256 characters
    // 'salt' must be between 0 and 64 hex digits
    // 'passowrd-hash' must be betwween 16 and 256 printable digits
    lazy_static! {
        static ref REGEX_HTBODY_AUTH_AV: Regex =
            Regex::new(r"^\{\{(?P<a>[[:alnum:]]{1,16})\}\}(?P<u>[[:alnum:]]{1,256}):(?P<l>\d):(?P<s>[[:xdigit:]]{0,64}):(?P<p>[[:print:]]{16,256})$")
                .unwrap();
    }
    // capture content using the pre-compiled regular expression
    REGEX_HTBODY_AUTH_AV.captures(acc)
}
