use std::env;
use std::fs::write;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use clap::Command as ClapCommand;
use clap::Arg;
use clap::ArgMatches;
use log::info;
use sysinfo::get_current_pid;
use crate::config::paths;

use crate::http::proxy_server;
use crate::http::proxy_server::start_caddy;
use crate::php::php_server;
use crate::php::php_server::PhpServerStartInput;
use crate::php::php_server::start_php_server;
use crate::utils::current_process_name;
use crate::utils::network::find_available_port;
use crate::utils::network::parse_default_port;
use crate::utils::project_directory::get_rymfony_project_directory;

const DEFAULT_PORT: &str = "8000";

pub(crate) fn command_config<'a>() -> ClapCommand<'a> {
    ClapCommand::new("server:start")
        .name("server:start")
        .alias("serve")
        .about("Runs an HTTP server")
        .after_help("
Runs an HTTP server and a PHP server (based on FPM or CGI depending on what's available).

Rymfony is capable of detecting your Document Root automatically.
It will do so in this order:
 * ./public/
 * ./web/

Rymfony is also capable of detecting your PHP entrypoint automatically.
It will do so in this order:
 * index.php
 * app_dev.php
 * app.php
")
        .arg(
            Arg::new("port")
                .long("port")
                .help("The TCP port to listen to")
                .default_value(DEFAULT_PORT.as_ref())
                .takes_value(true),
        )
        .arg(
            Arg::new("daemon")
                .short('d')
                .long("daemon")
                .help("Run the server in the background"),
        )
        .arg(
            Arg::new("document-root")
                .long("document-root")
                .help("Project's document root")
                .takes_value(true),
        )
        .arg(
            Arg::new("passthru")
                .long("passthru")
                .help("The PHP entrypoint all requests will be passed to")
                .takes_value(true),
        )
        .arg(
            Arg::new("no-tls")
                .long("no-tls")
                .help("Disable TLS. Use HTTP only."),
        )
        .arg(
            Arg::new("expose-server-header")
                .short('s')
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
    let rymfony_pid_file = paths::rymfony_pid_file();
    debug!("Looking for Rymfony PID file in \"{}\".",rymfony_pid_file.to_str().unwrap());

    if rymfony_pid_file.exists() {
        // Check if process is rymfony and exit if true.
        info!("The server is already running for this directory.");
        info!("Run the \"rymfony log\" command to tail its logs if you need.");
        return;
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
                    warn!("the best practice is to remove this file when deploying, and use \"app.php\" instead.");
                }
                script_filename = script.to_string();
                break;
            }
        }
    };

    let php_entrypoint_path = doc_root_path.join(script_filename.as_str());
    let (mut php_start_command, php_server_input) = if !php_entrypoint_path.is_file() {
        panic!("No PHP entrypoint specified.");
    } else {
        php_server::get_php_server_start_input()
    };

    let sapi = php_server_input.sapi;
    let sapi_string = sapi.to_string();

    if sapi_string == "unknown" {
        panic!("Unknown PHP SAPI to execute");
    } else {
        info!("PHP starting with module {}", sapi_string);
        info!("PHP entrypoint file: {}", &script_filename);
    }

    let php_port = php_server_input.port.clone();
    let php_bin = php_server_input.php_bin.clone();

    let mut php_process = start_php_server(&mut php_start_command, php_server_input.clone());

    info!("Starting Caddy HTTP server...");

    info!("Configured document root: {}", &document_root);

    let http_port = find_available_port(parse_default_port(
        args.value_of("port").unwrap_or(DEFAULT_PORT),
        DEFAULT_PORT,
    ));

    #[cfg(not(target_family = "windows"))]
    let rymfony_pid = get_current_pid().unwrap();
    #[cfg(target_family = "windows")]
    let rymfony_pid = get_current_pid().unwrap() as i32;

    write(&rymfony_pid_file, rymfony_pid.to_string()).expect("Could not write Rymfony PID to file.");

    //Serialize
    let no_tls = args.is_present("no-tls");

    // TODO: implement ".wip" (or other) custom domains.
    let host_name = "127.0.0.1".to_string();

    let (mut caddy_command, caddy_command_input) = proxy_server::get_caddy_start_command(
        !no_tls,
        host_name.clone(),
        http_port.clone(),
        php_port.clone(),
        document_root,
        script_filename,
        args.is_present("expose-server-header"),
    );

    let mut caddy_process = start_caddy(&mut caddy_command, caddy_command_input.config.clone());

    info!("Listening to {}://{}:{}", if no_tls { "http" } else { "https" }, host_name, http_port);

    ctrlc::set_handler(|| {
        crate::commands::stop::stop();
        info!("Bye! 🌙");
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    //
    // Healthcheck command
    //
    loop {
        let caddy_command_input = caddy_command_input.clone();

        std::thread::sleep(Duration::from_secs(100));

        //
        // PHP server healthcheck
        //
        let php_server_input = PhpServerStartInput {
            sapi,
            port: php_port,
            php_bin: php_bin.clone(),
        };
        let php_process_status = php_process.try_wait();
        match php_process_status {
            Ok(Some(status)) => {
                debug!("PHP stopped with exit code {}. Restarting it.", status.code().unwrap_or(255));
                php_process = start_php_server(&mut php_start_command, php_server_input);
                debug!("PHP restarted, running with PID {}", php_process.id());
            },
            Ok(None) => (), // PHP server still alive.
            Err(e) => panic!("An error occured when checking PHP server health: {:?}", e),
        };

        //
        // HTTP server healthcheck
        //
        let caddy_process_status = caddy_process.try_wait();
        match caddy_process_status {
            Ok(Some(status)) => {
                debug!("Caddy stopped with exit code {}. Restarting it.", status.code().unwrap_or(255));
                caddy_process = start_caddy(&mut caddy_command, caddy_command_input.config);
                debug!("Caddy restarted, running with PID {}", caddy_process.id());
            },
            Ok(None) => (), // HTTP server still alive.
            Err(e) => panic!("An error occured when checking Caddy HTTP server health: {:?}", e),
        };
    }
}

fn serve_background(args: &ArgMatches) {
    let port = find_available_port(parse_default_port(
        args.value_of("port").unwrap_or(DEFAULT_PORT),
        DEFAULT_PORT,
    ));

    let mut file_options = OpenOptions::new();
    file_options.read(true).append(true).write(true).create(true);

    let rymfony_log_file = file_options.open(paths::get_rymfony_process_log_file()).unwrap();
    let rymfony_err_file = file_options.open(paths::get_rymfony_process_err_file()).unwrap();

    let mut cmd = Command::new(current_process_name::get().as_str());
    cmd
        .stdout(Stdio::from(rymfony_log_file))
        .stderr(Stdio::from(rymfony_err_file))
        .arg("serve")
        .arg("--port")
        .arg(port.to_string())
    ;

    if args.is_present("no-tls") {
        cmd.arg("--no-tls");
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