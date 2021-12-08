use std::net::Ipv4Addr;
use std::{net::TcpListener, path::PathBuf};

use colored::*;
use structopt::StructOpt;
use warp::Filter;

const OBER_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ober",
    about = "a quick http server for whipping up local files."
)]
struct Opt {
    /// The port files will be served on
    #[structopt(short, long, env = "OBER_PORT")]
    port: Option<u16>,

    /// The address to bind to
    #[structopt(short, long, default_value = "0.0.0.0", env = "OBER_ADDRESS")]
    address: Ipv4Addr,

    /// Root folder
    #[structopt(parse(from_os_str), default_value = "./")]
    path: PathBuf,

    /// Surpess logs
    #[structopt(short, long)]
    silent: bool,
}

fn get_port(opt: &Opt) -> u16 {
    if let Some(port) = opt.port {
        port
    } else {
        (8080..65535)
            .find(|&port| TcpListener::bind((opt.address, port)).is_ok())
            .expect("could not find an available port")
    }
}

fn print_startup_info(opt: &Opt) {
    print!(
        "{}{}\n\n",
        "Starting ober, serving ".yellow(),
        opt.path.to_str().unwrap().blue(),
    );

    print!(
        "{}{}\n\n",
        "ober version: ".yellow(),
        OBER_VERSION.trim().blue()
    );
}

fn print_available_info(opt: &Opt, port: u16) {
    println!("{}", "Available on:".yellow());
    println!("  {}:{}", opt.address, port.to_string().green());
    print!("Hit CTRL-C to stop the server\n\n");
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let port = get_port(&opt);

    print_startup_info(&opt);

    let log = warp::log::custom(|info| {
        let now = chrono::Local::now();

        print!(
            "[{}] \"{} {} {:?}\" - \"{}\"",
            now.to_rfc3339(),
            info.method(),
            info.path(),
            info.version(),
            info.status()
        );

        if let Some(user_agent) = info.user_agent() {
            print!(" - \"{}\"", user_agent);
        }

        println!()
    });

    print_available_info(&opt, port);

    let filter = warp::fs::dir(opt.path).with(log);

    warp::serve(filter).run((opt.address.octets(), port)).await;
}
