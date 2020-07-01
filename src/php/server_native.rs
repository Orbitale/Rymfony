use std::process::Child;
use std::process::Command;

const CGI_DEFAULT_PORT: u16 = 65535;

pub(crate) fn start(php_bin: String) -> Child {
    let mut command = Command::new(php_bin);

    command
        .arg("-S")
        .arg(format!("127.0.0.1:{}", CGI_DEFAULT_PORT));

    if let Ok(child) = command.spawn() {
        println!("Running native PHP server with PID {}", child.id());

        return child;
    }

    panic!("Could not start native PHP server.");
}
