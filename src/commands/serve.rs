use std::fs::File;
use std::io::Write;
use std::process::Command;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use crate::http::proxy_server;
use crate::php::php_server;
use crate::php::php_server::PhpServerSapi;
use crate::utils::current_process_name;
use std::env;
use log::info;
use std::path::PathBuf;

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
        #[cfg(not(target_os = "windows"))]
        PhpServerSapi::FPM => "FPM",
        PhpServerSapi::CLI => "CLI",
        PhpServerSapi::CGI => "CGI",
    };
    info!("PHP started with module {}", sapi);

    info!("Starting HTTP server...");

    let port = args.value_of("port").unwrap_or(DEFAULT_PORT);
    let document_root = get_document_root(args.value_of("document-root").unwrap_or("").to_string());
    let script_filename = get_script_filename(&document_root, args.value_of("passthru").unwrap_or("index.php").to_string());

    info!("Configured document root: {}", &document_root);
    info!("PHP entrypoint file: {}", &script_filename);

    proxy_server::start(
        port.parse::<u16>().unwrap(),
        php_server.port(),
        &document_root,
        &script_filename
    );
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

fn get_script_filename(document_root: &str, script_filename_arg: String) -> String {
    let path = PathBuf::from(&script_filename_arg);

    if path.is_absolute() {
        debug!("Script path \"{}\" is absolute.", script_filename_arg);
        return script_filename_arg;
    }

    debug!("Script path \"{}\" is relative.", script_filename_arg);

    let mut path = PathBuf::from(document_root);
    path.push(&script_filename_arg);

    debug!("Relative script path \"{}\" resolved to \"{}\".", &script_filename_arg, path.to_str().unwrap());

    String::from(path.to_str().unwrap())
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
