use std::{net::TcpListener, path::PathBuf};

use structopt::StructOpt;
use warp::Filter;

use std::net::Ipv4Addr;

const SERRRV_VERSION: &str = env!("CARGO_PKG_VERSION");
const STYLE_INFO: ansi_term::Colour = ansi_term::Colour::Yellow;
const STYLE_VALUE: ansi_term::Colour = ansi_term::Colour::Blue;
const STYLE_OK: ansi_term::Colour = ansi_term::Colour::Green;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "serrrv",
    about = "a quick http server for whipping up local files."
)]
struct Opt {
    /// The port files will be served on
    #[structopt(short, long, env = "SERRRV_PORT")]
    port: Option<u16>,

    /// The address to bind to
    #[structopt(short, long, default_value = "0.0.0.0", env = "SERRRV_ADDRESS")]
    address: Ipv4Addr,

    /// Root folder
    #[structopt(parse(from_os_str), default_value = "./")]
    path: PathBuf,
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
        STYLE_INFO.paint("Starting serrrv, serving ").to_string(),
        STYLE_VALUE.paint(opt.path.to_str().unwrap()).to_string(),
    );

    print!(
        "{}{}\n\n",
        STYLE_INFO.paint("serrrv version: ").to_string(),
        STYLE_VALUE.paint(SERRRV_VERSION.trim()).to_string()
    );
}

fn print_available_info(opt: &Opt, port: u16) {
    println!("{}", STYLE_INFO.paint("Available on:").to_string(),);

    println!(
        "  {}:{}",
        opt.address,
        STYLE_OK.paint(port.to_string()).to_string()
    );

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
