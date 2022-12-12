use crate::php::structs::PhpServerSapi;
use crate::utils::project_directory::get_rymfony_project_directory;
use std::fs::create_dir_all;
use std::fs::File;
use std::process::Command;
use std::process::Stdio;

pub(crate) fn get_start_command(php_bin: String, port: &u16) -> (PhpServerSapi, Command) {
    let mut command = Command::new(php_bin);

    let log_path = get_rymfony_project_directory().unwrap().join("log");

    if !log_path.is_dir() {
        create_dir_all(&log_path).expect("Could not create log directory for project");
    }

    let error_log_file = log_path.join("php-cgi.log");

    if !error_log_file.exists() {
        let file_result = File::create(&error_log_file);
        if file_result.is_err() {
            warn!("Could not create php-cgi log file in {}", &error_log_file.to_str().unwrap());
        }
    }

    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("-b") // address:port
        .arg(format!("127.0.0.1:{}", port.to_string()))
        .arg("-d") // INI entries
        .arg(format!("error_log={}", error_log_file.to_str().unwrap()))
        .arg("-e") // extended information for debugger/profiler
    ;

    // Strangely, php-cgi stops after this amount of requests,
    // and it has no concurrency, so setting this to a high value
    // avoids having to restart php-cgi too much.
    command.env("PHP_FCGI_MAX_REQUESTS", "200000");

    (PhpServerSapi::CGI, command)
}
