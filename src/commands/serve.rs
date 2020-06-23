use std::convert::Infallible;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::process::Command;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;

use crate::utils::current_process_name;

const DEFAULT_LISTEN: &str = "127.0.0.1:5000";

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about("Runs an HTTP server")
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .help("The TCP socket to listen to, usually an IP with a Port")
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

#[tokio::main]
async fn serve_foreground(args: &ArgMatches) {
    pretty_env_logger::init();

    let listen = args.value_of("listen").unwrap_or(DEFAULT_LISTEN);

    let mut sockets = listen.to_socket_addrs().unwrap();

    let addr = SocketAddr::from(sockets.next().unwrap());

    let service_handler = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(request_handler))
    });

    println!("Server listening to http://{}", addr);

    let server = Server::bind(&addr).serve(service_handler);

    server
        .await
        .expect("An error occured when starting the server");
}

fn serve_background(args: &ArgMatches) {
    let listen = args.value_of("listen").unwrap_or(DEFAULT_LISTEN);

    let subprocess = Command::new(current_process_name::get().as_str())
        .arg("serve")
        .arg("--listen")
        .arg(listen)
        .spawn()
        .expect("Failed to start server as a background process");

    let pid = subprocess.id();
    let mut file = File::create(".pid").expect("Cannot create PID file");
    file.write_all(pid.to_string().as_ref())
        .expect("Cannot write to PID file");

    println!("Background server running with PID {}", pid);
}

async fn request_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", _req.method(), _req.uri());

    Ok(Response::new("Hello, World".into()))
}
