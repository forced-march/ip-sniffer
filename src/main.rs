use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::process;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

const MAX: u16 = 65535;
/*
ip_sniffer.exe -h
ip_sniffer.exe -j 100 192.168.1.1
ip_sniffer.exe 192.168.1.1
*/

struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16,
}
const DEFAULT_NUMBER_OF_THREADS: u16 = 4;
fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        };
        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}
impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("Too many arguments");
        }
        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                flag: String::from(""),
                ipaddr,
                threads: DEFAULT_NUMBER_OF_THREADS,
            });
        } else {
            let flag = args[1].clone();
            if (flag.contains("-h") || flag.contains("-help") && args.len() == 2) {
                println!(
                    "Usage: -j to select how many threads you want
                \r\n      -h or -help tp show this help message"
                );
                return Err("help");
            } else if flag.contains("-h") || flag.contains("-help") {
                return Err("Too many arguments");
            } else if flag.contains("-j") {
                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("not a vaild IPADDR; must be IPv4 or IPv6"),
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(n) => n,
                    Err(_) => return Err("Failed to pass thread number"),
                };
                return Ok(Arguments {
                    threads,
                    flag,
                    ipaddr,
                });
            } else {
                return Err("Invalid syntax");
            }
        }
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    for arg in &args {
        println!("{}", arg);
    }
    println!("{:?}", args);
    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} problem parsing arguments: {}", program, err);
            process::exit(0);
        }
    });
    let num_threads = arguments.threads;
    let addr = arguments.ipaddr;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }
    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }
    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}
