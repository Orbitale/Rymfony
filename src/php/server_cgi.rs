use crate::php::php_server::PhpServer;
use crate::php::structs::PhpServerSapi;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::fs::File;
use crate::utils::project_folder::get_rymfony_project_directory;

const CGI_DEFAULT_PORT: u16 = 65535;

pub(crate) fn start(php_bin: String) -> (PhpServer, Child) {
    let mut command = Command::new(php_bin);

    let error_log_file = get_rymfony_project_directory().unwrap()
        .join("php-cgi.log")
    ;

    if !error_log_file.exists() {
        let file_result = File::create(&error_log_file);
        if file_result.is_err() {
            warn!("Could not create php-cgi log file in {}", &error_log_file.to_str().unwrap());
        }
    }

    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("-b")
        .arg(format!("127.0.0.1:{}", CGI_DEFAULT_PORT))
        .arg("-d")
        .arg(format!("error_log={}", error_log_file.to_str().unwrap()))
        .arg("-e")
    ;

    if let Ok(child) = command.spawn() {
        info!("Running php-cgi with PID {}", child.id());

        return (PhpServer::new(CGI_DEFAULT_PORT, PhpServerSapi::CGI), child);
    }

    panic!("Could not start php-cgi.");
}
