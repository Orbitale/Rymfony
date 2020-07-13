use std::fs::File;
use std::io::Write;
use std::process::Command;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use crate::http::proxy_server;
use crate::php::php_server;
use crate::utils::current_process_name;

const DEFAULT_PORT: &str = "8000";

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about("Runs an HTTP server")
        .arg(
            Arg::with_name("port")
                .long("port")
                .help("The TCP port to listen to")
                .default_value(DEFAULT_PORT.as_ref())
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

    info!("Starting PHP...");

    php_server::start();

    info!("Starting HTTP server...");

    let port = args.value_of("port").unwrap_or(DEFAULT_PORT);
    proxy_server::start(port.parse::<u16>().unwrap());
}

fn serve_background(args: &ArgMatches) {
    let subprocess = Command::new(current_process_name::get().as_str())
        .arg("serve")
        .arg("--port")
        .arg(args.value_of("port").unwrap_or(DEFAULT_PORT))
        .spawn()
        .expect("Failed to start server as a background process");

    let pid = subprocess.id();
    let mut file = File::create(".pid").expect("Cannot create PID file");
    file.write_all(pid.to_string().as_ref())
        .expect("Cannot write to PID file");

    info!("Background server running with PID {}", pid);
}
