use crate::utils::current_process_name;
use std::env;
use std::fs::File;
use std::process::Command;
use std::process::Stdio;
use clap::SubCommand;
use clap::App;
use clap::Arg;
use clap::ArgMatches;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use hyper::Response;
use hyper::Request;
use hyper::Body;
use hyper::Server;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use std::io::Write;

const DEFAULT_LISTEN: &str = "127.0.0.1:5000";

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about("Runs an HTTP server")
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .help("The TCP socket to listen to, usually an IP with a Port")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("daemon")
                .short("d")
                .long("daemon")
                .help("Run the server in the background")
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
async fn serve_foreground(args: &ArgMatches) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let listen = args.value_of("listen").unwrap_or(DEFAULT_LISTEN);

    println!("Serving {}", listen);

    let mut sockets = listen.to_socket_addrs().unwrap();

    let addr = SocketAddr::from(sockets.next().unwrap());

    let service_handler = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(request_handler))
    });

    println!("Serving on http://{}", addr);

    let server = Server::bind(&addr).serve(service_handler);

    server.await?;

    Ok(())
}

fn serve_background(args: &ArgMatches) {
    println!("Serving in background");

    let listen = args.value_of("listen").unwrap_or(DEFAULT_LISTEN);

    println!("Serving {}", listen);

    let subprocess = Command::new(current_process_name::get().as_str())
        .arg("serve")
        .arg("--listen")
        .arg(listen)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to start server as a background process")
    ;

    let pid = subprocess.id();
    let mut file = File::create(".pid").expect("Cannot write to PID file");
    file.write_all(pid.to_string().as_ref());

}

async fn request_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{} {}", _req.method(), _req.uri());

    Ok(Response::new("Hello, World".into()))
}
