use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::process::Command;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use crate::http::proxy_server;
use crate::utils::current_process_name;
use crate::utils::php_server;

const DEFAULT_LISTEN: &str = "127.0.0.1:8000";

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about("Runs an HTTP server")
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .help("The TCP socket to listen to, usually an IP with a Port")
                .default_value(DEFAULT_LISTEN)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("daemon")
                .short("d")
                .long("daemon")
                .help("Run the server in the background"),
        )
}

pub(crate) fn serve(args: &ArgMatches) {
    if args.is_present("daemon") {
        serve_background(args);
    } else {
        serve_foreground(args);
    }
}

fn serve_foreground(args: &ArgMatches) {
    pretty_env_logger::init();

    println!("Starting PHP...");

    php_server::start();

    let listen = args.value_of("listen").unwrap_or(DEFAULT_LISTEN);
    let mut sockets = listen.to_socket_addrs().unwrap();
    let addr = SocketAddr::from(sockets.next().unwrap());

    println!("Starting HTTP server...");

    proxy_server::start(addr);
}

fn serve_background(args: &ArgMatches) {
    let subprocess = Command::new(current_process_name::get().as_str())
        .arg("serve")
        .arg("--listen")
        .arg(args.value_of("listen").unwrap_or(DEFAULT_LISTEN))
        .spawn()
        .expect("Failed to start server as a background process");

    let pid = subprocess.id();
    let mut file = File::create(".pid").expect("Cannot create PID file");
    file.write_all(pid.to_string().as_ref())
        .expect("Cannot write to PID file");

    println!("Background server running with PID {}", pid);
}
