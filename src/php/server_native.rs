use std::process::{Command, Child};
use crate::php::php_server::{PhpServer, PhpServerSapi};

const NATIVE_DEFAULT_PORT: u16 = 65535;

pub(crate) fn start(php_bin: String) -> (PhpServer, Child) {
    let mut command = Command::new(php_bin);

    command
        .arg("-S")
        .arg(format!("127.0.0.1:{}", NATIVE_DEFAULT_PORT));

    if let Ok(child) = command.spawn() {
        println!("Running native PHP server with PID {}", child.id());

        return (PhpServer::new(NATIVE_DEFAULT_PORT, PhpServerSapi::CLI), child);
    }

    panic!("Could not start native PHP server.");
}
