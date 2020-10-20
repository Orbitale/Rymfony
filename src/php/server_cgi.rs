use crate::php::php_server::PhpServer;
use crate::php::structs::PhpServerSapi;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

const CGI_DEFAULT_PORT: u16 = 65535;

pub(crate) fn start(php_bin: String) -> (PhpServer, Child) {
    let mut command = Command::new(php_bin);

    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("-b")
        .arg(format!("127.0.0.1:{}", CGI_DEFAULT_PORT));

    if let Ok(child) = command.spawn() {
        info!("Running php-cgi with PID {}", child.id());

        return (PhpServer::new(CGI_DEFAULT_PORT, PhpServerSapi::CGI), child);
    }

    panic!("Could not start php-cgi.");
}
