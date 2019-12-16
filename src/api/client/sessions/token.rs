//! Client API - session token module

// std
use std::time::SystemTime;

// rand
use rand::Rng;

// hmac
use hmac::{Hmac, Mac};

// sha3
use sha3::Sha3_256;

// config
use crate::config;

/// SessionTroken structure
pub struct SessionToken {
    user: String,
    ts_since: u64,
    ts_valid: u64,
    nonce: u64,
    token: String,
}

/// SessionToken implementation
impl SessionToken {
    // new() method
    pub fn new() -> SessionToken {
        SessionToken {
            user: "null".to_string(),
            ts_since: 0,
            ts_valid: 0,
            nonce: 0,
            token: "null".to_string(),
        }
    }
    // set_user() setter
    pub fn set_user(&mut self, user: String) {
        self.user = user;
    }
    // set_tssince() setter
    pub fn set_tssince(&mut self, ts: u64) {
        self.ts_since = ts;
    }
    // set_validfor() method
    pub fn set_validfor(&mut self, duration: u64) -> u64 {
        self.ts_valid = self.ts_since + duration;
        self.ts_valid
    }
    // set_nonce() setter
    pub fn set_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }
    // set_token() setter
    pub fn set_token(&mut self, token: String) {
        self.token = token;
    }
    // gen_token() method
    pub fn gen_token(&mut self, cfg: &config::CConfig) -> std::io::Result<()> {
        // get current system time (in seconds since the Unix Epoch)
        let now = SystemTime::now();
        let time = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // set current time to 'since' timestamp
        self.ts_since = time;
        // generate a random number of 64 bits
        let mut rng = rand::thread_rng();
        let nonce: u64 = rng.gen();
        // set the nonce
        self.nonce = nonce;
        // concatenate the user and time with the nonce
        let utn = format!("{}{}{}", self.user, self.ts_since, self.nonce);
        // hash the above elements
        let secret = cfg.api.as_ref().unwrap().secret();
        let token = gen_hmac_string(&utn, secret);
        // set the hashed token
        self.token = token;
        // confirm completion
        Ok(())
    }
    // user() method
    pub fn user(&self) -> String {
        self.user.clone()
    }
    // ts_since() method
    pub fn ts_since(&self) -> String {
        format!("{}", self.ts_since)
    }
    // nonce() method
    pub fn nonce(&self) -> String {
        format!("{}", self.nonce)
    }
    // token() method
    pub fn token(&self) -> String {
        self.token.clone()
    }
    // validate() method
    // check the integrity of the token (hash)
    pub fn validate(&self, cfg: &config::CConfig) -> Option<String> {
        // concatenate the user and time with the nonce
        let utn = format!("{}{}{}", self.user, self.ts_since, self.nonce);
        // hash the above elements
        let secret = cfg.api.as_ref().unwrap().secret();
        let token = gen_hmac_string(&utn, secret);
        // do a comparison betwen the stored and the recompute tokens
        if self.token == token {
            Some(token)
        } else {
            None
        }
    }
}

// type alias
type HmacSha3_256 = Hmac<Sha3_256>;

// gen_hmac_string() function
fn gen_hmac_string(input: &String, secret: String) -> String {
    let mut mac = HmacSha3_256::new_varkey(secret.as_bytes()).expect("invalid key length");
    mac.input(input.as_bytes());
    let res = mac.result();
    let out = res.code();

    format!("{:x}", out)
}
