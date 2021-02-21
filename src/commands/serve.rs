use std::env;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use log::info;
use sysinfo::get_current_pid;
use sysinfo::ProcessExt;
use sysinfo::SystemExt;

use crate::http::proxy_server;
use crate::php::php_server;
use crate::php::php_server::PhpServer;
use crate::php::structs::PhpServerSapi;
use crate::php::structs::ServerInfo;
use crate::utils::current_process_name;
use crate::utils::network::find_available_port;
use crate::utils::network::parse_default_port;
use crate::utils::project_directory::get_rymfony_project_directory;

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
    let path = get_rymfony_project_directory().unwrap();
    let rymfony_pid_file = path.join("rymfony.pid");
    debug!(
        "Looking for PID file in \"{}\".",
        rymfony_pid_file.to_str().unwrap()
    );
    if rymfony_pid_file.exists() {
        // Check if process is rymfony and exit if true.

        let infos: ServerInfo =
            serde_json::from_str(read_to_string(&rymfony_pid_file).unwrap().as_str())
                .expect("Unable to unserialize data from PID file.");

        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        for (pid, proc_) in system.get_processes() {
            #[cfg(not(target_family = "windows"))]
            let process_pid = *pid;

            #[cfg(target_family = "windows")]
            let process_pid = *pid as i32;

            let mut pname = proc_.exe().to_str().unwrap();
            let pname_lower = pname.to_lowercase();
            pname = pname_lower.as_str();

            let exe_rymfony_name = if cfg!(not(target_family = "windows")) {
                "rymfony"
            } else {
                "rymfony.exe"
            };

            if &process_pid == &infos.pid() && pname.ends_with(exe_rymfony_name) {
                info!(
                    "The server is already running and listening to {}://127.0.0.1:{}",
                    infos.scheme(),
                    infos.port()
                );
                return;
            }
        }
    }

    let mut document_root =
        get_document_root(args.value_of("document-root").unwrap_or("").to_string());
    if document_root.ends_with('/') {
        document_root.pop();
    }
    if document_root.ends_with('\\') {
        document_root.pop();
    }
    document_root.push_str(if cfg!(target_family = "windows") {
        "\\"
    } else {
        "/"
    });
    let doc_root_path = PathBuf::from(document_root.as_str());
    let common_scripts_names = vec!["index.php", "app_dev.php", "app.php"];
    let mut script_filename = "index.php".to_string();
    if args.is_present("passthru") {
        script_filename = args.value_of("passthru").unwrap_or("index.php").to_string()
    } else {
        for script in common_scripts_names {
            let php_entrypoint_path = doc_root_path.join(script);
            if php_entrypoint_path.is_file() {
                if script == "app_dev.php" {
                    warn!("Entrypoint was automaticaly resolved to \"app_dev.php\".");
                    warn!("If you are using Rymfony on productions servers,");
                    warn!("the best practice is to remove this file when deploying, and us \"app.php\" instead.");
                }
                script_filename = script.to_string();
                break;
            }
        }
    };

    let php_entrypoint_path = doc_root_path.join(script_filename.as_str());
    let php_server = if !php_entrypoint_path.is_file() {
        warn!("No PHP entrypoint file");
        PhpServer::new(0, PhpServerSapi::Unknown)
    } else {
        info!("Starting PHP...");

        php_server::start()
    };

    let sapi = match php_server.sapi() {
        PhpServerSapi::FPM => "FPM",
        PhpServerSapi::CLI => "CLI",
        PhpServerSapi::CGI => "CGI",
        PhpServerSapi::Unknown => "?",
    };
    if sapi == "?" {
        info!("Skip PHP start");
    } else {
        info!("PHP started with module {}", sapi);
        info!("PHP entrypoint file: {}", &script_filename);
    }

    info!("Starting HTTP server...");

    info!("Configured document root: {}", &document_root);

    let port = find_available_port(parse_default_port(
        args.value_of("port").unwrap_or(DEFAULT_PORT),
        DEFAULT_PORT,
    ));

    #[cfg(not(target_family = "windows"))]
    let pid = get_current_pid().unwrap();
    #[cfg(target_family = "windows")]
    let pid = get_current_pid().unwrap() as i32;

    let args_str: Vec<String> = Vec::new();
    let scheme = if args.is_present("no-tls") {
        "http".to_string()
    } else {
        "https".to_string()
    };
    let pid_info = ServerInfo::new(
        pid,
        port,
        scheme,
        "Web Server".to_string(),
        current_process_name::get(),
        args_str,
    );

    //Serialize
    let serialized = serde_json::to_string_pretty(&pid_info).unwrap();
    let mut versions_file = File::create(&rymfony_pid_file).unwrap();

    versions_file
        .write_all(serialized.as_bytes())
        .expect("Could not write Process informations to JSON file.");

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
    let port = find_available_port(parse_default_port(
        args.value_of("port").unwrap_or(DEFAULT_PORT),
        DEFAULT_PORT,
    ));

    let mut cmd = Command::new(current_process_name::get().as_str());
    cmd.arg("serve").arg("--port").arg(port.to_string());

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
    let project_directory =
        get_rymfony_project_directory().expect("Unable to get Rymfony directory for this project");

    let mut file = File::create(project_directory.join(".pid")).expect("Cannot create PID file");
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
