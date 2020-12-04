use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use log::info;

use crate::http::proxy_server;
use crate::php::php_server;
use crate::php::structs::PhpServerSapi;
use crate::utils::current_process_name;
use crate::utils::network::find_available_port;
use crate::utils::network::parse_default_port;

const DEFAULT_PORT: &str = "8000";

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("server:start")
        .name("server:start")
        .alias("serve")
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
        .arg(
            Arg::with_name("document-root")
                .long("document-root")
                .help("Project's document root")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("passthru")
                .long("passthru")
                .help("The PHP script all requests will be passed to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-tls")
                .long("no-tls")
                .help("Disable TLS. Use HTTP only."),
        )
        .arg(
            Arg::with_name("allow-http")
                .long("allow-http")
                .help("Do not redirect HTTP request to HTTPS"),
        )
        .arg(
            Arg::with_name("expose-server-header")
                .short("s")
                .long("expose-server-header")
                .help("Add server header into all response"),
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
    info!("Starting PHP...");

    let php_server = php_server::start();

    let sapi = match php_server.sapi() {
        PhpServerSapi::FPM => "FPM",
        PhpServerSapi::CLI => "CLI",
        PhpServerSapi::CGI => "CGI",
        PhpServerSapi::Unknown => "?",
    };
    info!("PHP started with module {}", sapi);

    info!("Starting HTTP server...");

    let port = find_available_port(parse_default_port(args.value_of("port").unwrap_or(DEFAULT_PORT), DEFAULT_PORT));

    let document_root = get_document_root(args.value_of("document-root").unwrap_or("").to_string());
    let script_filename = args.value_of("passthru").unwrap_or("index.php").to_string();

    info!("Configured document root: {}", &document_root);
    info!("PHP entrypoint file: {}", &script_filename);

    proxy_server::start(
        !args.is_present("no-tls"),
        !args.is_present("allow-http"),
        port,
        php_server.port(),
        document_root,
        script_filename,
        args.is_present("expose-server-header"),
    );

    unreachable!();
}

fn serve_background(args: &ArgMatches) {
    let port = find_available_port(parse_default_port(args.value_of("port").unwrap_or(DEFAULT_PORT), DEFAULT_PORT));

    let mut cmd = Command::new(current_process_name::get().as_str());
        cmd.arg("serve")
        .arg("--port")
        .arg(port.to_string());

    if args.is_present("no-tls") {
        cmd.arg("--no-tls");
    }
    if args.is_present("allow-http") {
        cmd.arg("--allow-http");
    }
    if args.is_present("expose-server-header") {
        cmd.arg("--expose-server-header");
    }
    if args.is_present("document-root") {
        cmd.arg("--document-root")
            .arg(args.value_of("document-root").unwrap_or("").to_string());
    }
    if args.is_present("passthru") {
        cmd.arg("--passthru")
            .arg(args.value_of("passthru").unwrap_or("index.php").to_string());
    }

    let subprocess = cmd
        .spawn()
        .expect("Failed to start server as a background process");

    let pid = subprocess.id();
    let mut file = File::create(".pid").expect("Cannot create PID file");
    file.write_all(pid.to_string().as_ref())
        .expect("Cannot write to PID file");

    info!("Background server running with PID {}", pid);
}

fn get_document_root(document_root_arg: String) -> String {
    let path = PathBuf::from(&document_root_arg);

    if path.is_absolute() {
        return document_root_arg;
    }

    let document_root = if document_root_arg == "" {
        autodetect_document_root()
    } else {
        PathBuf::from(document_root_arg)
    };

    String::from(document_root.to_str().unwrap())
}

fn autodetect_document_root() -> PathBuf {
    let current_dir = env::current_dir().unwrap();

    // {cwd}/public/ , usually recent projects
    let mut public_dir = PathBuf::from(&current_dir);
    public_dir.push("public/");
    if public_dir.is_dir() {
        return public_dir;
    }

    // {cwd}/web/ , symfony 2 style
    let mut web_dir = PathBuf::from(&current_dir);
    web_dir.push("web/");
    if web_dir.is_dir() {
        return web_dir;
    }

    current_dir
}
