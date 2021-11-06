use std::fs::write;

use crate::php::binaries;
use crate::php::server_cgi::start as start_cgi;
use crate::php::server_fpm::start as start_fpm;
use crate::php::structs::PhpServerSapi;
use crate::php::structs::ServerInfo;
use crate::utils::project_directory::clean_rymfony_runtime_files;
use crate::utils::stop_process;

use is_executable::IsExecutable;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time;
use crate::config::config::php_server_pid;
use crate::config::paths::php_server_pid_file;
use crate::config::paths::rymfony_server_info_file;
use crate::utils::network::find_available_port;

const PHP_DEFAULT_PORT: u16 = 60000;

pub(crate) fn start() -> (PhpServerSapi, u16) {
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

    let (php_server_sapi, mut command) = start_php_server(&php_bin, &php_server_port);

    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);

    let mut process = command.spawn().expect("Could not start PHP server.");
    let process_status = process.try_wait();

    match process_status {
        Ok(Some(status)) => panic!("PHP server exited with {}", status),
        Ok(None) => {
            info!("PHP server is ready and listening to port {}", &php_server_port);
        }
        Err(e) => panic!("An error occured when checking PHP server health: {:?}", e),
    }

    let process_pid = process.id();

    let args_str: Vec<String> = Vec::new();
    let process_pid_string = if process_pid > 0 {
        process_pid.to_string()
    } else {
        panic!("Could not retrieve PHP server's PID. Maybe the server has failed to start, or stopped right after starting.");
    };

    write(php_server_pid_file(), &process_pid_string).expect("Could not write PHP server PID to file.");

    ctrlc::set_handler(move || {
        info!("Stopping PHP process...");

        let pid = php_server_pid();
        stop_process::stop(&pid); // Stop fpm children
        info!("PHP process stopped.");

        clean_rymfony_runtime_files();
        info!("Cleaned Rymfony runtime files.");
    })
    .expect("Error setting Ctrl-C handler");

    let pid_info = ServerInfo::new(
        php_server_port,
        "".to_string(),
        format!("{}", php_server_sapi),
        php_bin.clone(),
        args_str,
    );

    std::thread::spawn(move || {
        trace!("Starting healthcheck loop.");
        let mut server_process = process;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let process_status = server_process.try_wait();
            match process_status {
                Ok(Some(status)) => {
                    debug!("PHP stopped with exit code {}. Restarting it.", status.code().unwrap_or(255));
                    let (_, mut command) = start_php_server(&php_bin, &php_server_port);
                    server_process = command.spawn().expect("Could not restart PHP server after failure.");
                    let pid = server_process.id().to_string();
                    write(php_server_pid_file(), &pid).expect("Could not write PHP server PID to file after failure.");
                    debug!("PHP restarted, running with PID {}", pid);
                },
                Ok(None) => (), // PHP server still alive.
                Err(e) => panic!("An error occured when checking PHP server health: {:?}", e),
            };
        }
    });

    // Serialize Server info
    let serialized = serde_json::to_string_pretty(&pid_info).unwrap();

    write(rymfony_server_info_file(), serialized.as_bytes())
        .expect("Could not write PHP process information to cache file.");

    (php_server_sapi, php_server_port)
}

fn start_php_server(php_bin: &String, port: &u16) -> (PhpServerSapi, Command) {
    let (sapi, command) =
        if php_bin.contains("-fpm") && cfg!(not(target_family = "windows")) {
            start_fpm(php_bin.clone(), port)
        } else if php_bin.contains("-cgi") {
            start_cgi(php_bin.clone(), port)
        } else {
            panic!("Rymfony only supports PHP-FPM (linux) and PHP-CGI (Windows), and none of these SAPIs was found.");
        }
    ;

    (sapi, command)
}
