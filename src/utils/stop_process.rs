use std::process::Command;
use std::process::Stdio;

#[cfg(target_family = "windows")]
pub(crate) fn stop(pid: &str) {
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

#[cfg(not(target_family = "windows"))]
pub(crate) fn stop(pid: &str) {
    let mut child = Command::new("kill")
        .arg(pid)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("Could not stop server.");

    child
        .wait()
        .expect("An error occured when trying to stop the server");
}
