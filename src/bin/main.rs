//! # rVRRPd
//!
//! `rVRRPd` is aimed to be a fast, secure and multi-platform VRRPv2 implementation.
extern crate rVRRPd;
use rVRRPd::{listen_ip_pkts, Config};

// getopts
use getopts::Options;

// std
use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ctrlc (linux signal handling)
extern crate ctrlc;

/// MyError Type
#[derive(Debug)]
struct MyError(String);

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}
impl Error for MyError {}

// print_usage() function
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} -m0|1 [options]>", program);
    print!("{}", opts.usage(&brief));
}

// parse_cl_opts() function
fn parse_cli_opts(args: &[String]) -> Result<Config, Box<dyn Error>> {
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optflag("h", "help", "display help information");
    opts.optopt("i", "iface", "ethernet interface to listen on", "INTERFACE");
    opts.optopt(
        "m",
        "mode",
        "operation mode:\n 0(sniff), 1(foreground)",
        "MODE",
    );
    opts.optopt("c", "conf", "path to configuration file", "FILE");
    opts.optopt(
        "d",
        "debug",
        "debugging level:\n0(none), 1(low), 2(medium), 3(high), 5(extensive)",
        "LEVEL",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => return Result::Err(Box::new(MyError(f.to_string().into()))),
    };

    // help command-line option
    if matches.opt_present("help") || args[1..].is_empty() {
        print_usage(&program, opts);
        std::process::exit(1);
    }

    // mode command-line option
    let mode = matches.opt_str("mode");
    let mode = match mode {
        Some(x) => x.parse::<u8>().unwrap(),
        None => {
            return Result::Err(Box::new(MyError("No operation mode specified (-m)".into())));
        }
    };

    // iface command-line option
    let iface = matches.opt_str("iface");
    let iface = match iface {
        Some(x) => Option::Some(x.parse::<String>().unwrap()),
        None => {
            if mode == 0 {
                return Result::Err(Box::new(MyError("No interface specified (-i)".into())));
            }
            Option::None
        }
    };

    // config command-line option
    let conf = matches.opt_str("conf");
    let conf = match conf {
        Some(x) => Option::Some(x.parse::<String>().unwrap()),
        None => {
            // if mode == 1 {
            //     return Result::Err(Box::new(MyError(
            //         "No configuration file specified (-c)".into(),
            //     )));
            // }
            Option::None
        }
    };

    // debug level command-line option
    let debug = matches.opt_str("debug");
    let debug = match debug {
        Some(x) => Option::Some(x.parse::<u8>().unwrap()),
        None => Some(0),
    };

    Ok(Config::new(iface, mode, conf, debug))
}

// run() function
fn run(cfg: Config) -> Result<(), Box<dyn Error>> {
    // print information
    println!("Starting rVRRPd");

    // Thread safe shared variable
    let shutdown = Arc::new(AtomicBool::new(false));

    // Set up SIGINT signal handler
    let shutdown_c1 = Arc::clone(&shutdown);
    ctrlc::set_handler(move || {
        println!("\nReceived CTRL-C (SIGINT)");
        shutdown_c1.swap(true, Ordering::Relaxed);
    })
    .expect("Error while setting SIGINT signal handler.");

    // Listen to IP packets
    match listen_ip_pkts(&cfg, shutdown) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Result::Err(Box::new(MyError(
                format!("An error occured while starting VRRP: {}", e).into(),
            )));
        }
    }
}

// main() function
fn main() {
    let args: Vec<String> = env::args().collect();

    match parse_cli_opts(&args) {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        Ok(c) => match run(c) {
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
            Ok(_) => {
                std::process::exit(0);
            }
        },
    }
}
