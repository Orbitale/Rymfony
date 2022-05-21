use std::fs::write;

use crate::php::binaries;
use crate::php::server_cgi::get_start_command as get_cgi_start_command;
use crate::php::server_fpm::get_start_command as get_fpm_start_command;
use crate::php::structs::PhpServerSapi;

use is_executable::IsExecutable;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::thread;
use std::time;
use crate::config::paths::php_server_pid_file;
use crate::utils::network::find_available_port;

const PHP_DEFAULT_PORT: u16 = 60000;

#[derive(Clone)]
pub(crate) struct PhpServerStartInput {
    pub(crate) sapi: PhpServerSapi,
    pub(crate) port: u16,
    pub(crate) php_bin: String,
}

pub(crate) fn get_php_server_start_input() -> (Command, PhpServerStartInput) {
    let php_bin = binaries::get_project_version();

    let phpbin_path = PathBuf::from(php_bin.as_str());

    if !phpbin_path.is_executable() {
        error!(
            "PHP binary not found or not executable: {}",
            php_bin.as_str()
        );
        error!("You can execute \"rymfony php:list --refresh\" to update binaries paths cache.");
        panic!("Unable to start the required PHP binary");
    }

    let php_server_port = find_available_port(PHP_DEFAULT_PORT);

    let (php_server_sapi, command) = get_php_server_start_command(&php_bin, &php_server_port);

    (command, PhpServerStartInput {
        sapi: php_server_sapi,
        port: php_server_port,
        php_bin: String::from(php_bin),
    })
}

fn get_php_server_start_command(php_bin: &String, port: &u16) -> (PhpServerSapi, Command) {
    let (sapi, command) =
        if php_bin.contains("-fpm") && cfg!(not(target_family = "windows")) {
            get_fpm_start_command(php_bin.clone(), port)
        } else if php_bin.contains("-cgi") {
            get_cgi_start_command(php_bin.clone(), port)
        } else {
            panic!("Rymfony only supports PHP-FPM (linux) and PHP-CGI (Windows), and none of these SAPIs was found.");
        }
    ;

    (sapi, command)
}

pub(crate) fn start_php_server(command: &mut Command, input: PhpServerStartInput) -> Child {
    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);

    let mut process = command.spawn().expect("Could not start PHP server.");
    let process_status = process.try_wait();

    match process_status {
        Ok(Some(status)) => panic!("PHP server exited with {}", status),
        Ok(None) => {
            info!("PHP server is ready and listening to port {}", &input.port);
        }
        Err(e) => panic!("An error occured when checking PHP server health: {:?}", e),
    }

    let process_pid = process.id();

    let process_pid_string = if process_pid > 0 {
        process_pid.to_string()
    } else {
        panic!("Could not retrieve PHP server's PID. Maybe the server has failed to start, or stopped right after starting.");
    };

    write(php_server_pid_file(), &process_pid_string).expect("Could not write PHP server PID to file.");

    process
}