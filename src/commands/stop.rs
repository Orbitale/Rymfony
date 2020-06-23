use clap::App;
use clap::SubCommand;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("stop").about("Stops a potentially running HTTP server")
}

pub(crate) fn stop() {
    if Path::new(".pid").exists() {
        let pid = fs::read_to_string(".pid").unwrap();
        stop_process(pid.as_ref());
        println!("Stopped server running with PID {}", pid);
        fs::remove_file(".pid").expect("Could not remove the PID file")
    } else {
        println!("Seems like server is not running");
    }
}

#[cfg(target_os = "windows")]
fn stop_process(pid: &str) {
    let mut child = Command::new("taskkill")
        .arg("/T") // Stops process tree
        .arg("/F") // Force stop
        .arg("/PID")
        .arg(pid)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Could not stop server.");

    child
        .wait()
        .expect("An error occured when trying to stop the server");
}

#[cfg(not(target_os = "windows"))]
fn stop_process(pid: &str) {
    let mut child = Command::new("kill")
        .arg("-9") // SIGKILL
        .arg(pid)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Could not stop server.");

    child
        .wait()
        .expect("An error occured when trying to stop the server");
}
