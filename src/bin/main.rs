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

// ctrlc (linux signal handling)
extern crate ctrlc;

/// MyError Type
#[derive(Debug)]
struct MyError(String);

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for MyError {}

// print_usage() function
fn print_usage(program: &str, opts: Options) {
    let modes = format!(
        "\
    Modes:
    0 = VRRPv2 Sniffer
    1 = VRRPv2 Virtual Router (foreground)
    2 = VRRPv2 Virtual Router (daemon)\
    "
    );
    let usage = format!("Usage: {} -m0|1|2 [options]\n\n{}", program, modes);
    print!("{}", opts.usage(&usage));
}

// parse_cl_opts() function
fn parse_cli_opts(args: &[String]) -> Result<Config, Box<dyn Error>> {
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optflag("h", "help", "display help information");
    opts.optopt(
        "i",
        "iface",
        "ethernet interface to listen on (sniffer mode)",
        "INTERFACE",
    );
    opts.optopt(
        "m",
        "mode",
        "operation modes (see Modes):\n 0(sniffer), 1(foreground), 2(daemon)",
        "MODE",
    );
    opts.optopt(
        "c",
        "conf",
        "path to configuration file:\n (default to /etc/rvrrpd/rvrrpd.conf)",
        "FILE",
    );
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
        None => Option::None,
    };

    // debug level command-line option
    let debug = matches.opt_str("debug");
    let debug = match debug {
        Some(x) => Option::Some(x.parse::<u8>().unwrap()),
        None => None,
    };

    Ok(Config::new(iface, mode, conf, debug))
}

// run() function
fn run(cfg: Config) -> Result<(), Box<dyn Error>> {
    // print information
    println!("Starting rVRRPd");

    // Listen to IP packets
    match listen_ip_pkts(&cfg) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Result::Err(Box::new(MyError(
                format!("A runtime error occured: {}", e).into(),
            )));
        }
    }
}

// main() function
fn main() {
    let args: Vec<String> = env::args().collect();

    match parse_cli_opts(&args) {
        // error while parsing cli options
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        // if a configuration is returned from the parser
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
