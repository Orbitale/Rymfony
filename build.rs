use std::process::Command;
use std::process::Stdio;
use std::path::Path;
use std::fs::File;
use std::fs::remove_file;

#[cfg(target_os="windows")]
mod config {
    pub(crate) const CADDY_BIN_FILE: &'static str = "caddy.exe";
    pub(crate) const SHELL_TO_EXEC: &'static str = "powershell.exe";
    pub(crate) const DOWNLOAD_CADDY_SCRIPT: &'static str = "download_caddy.ps1";
}

#[cfg(not(target_os="windows"))]
mod config {
    pub(crate) const CADDY_BIN_FILE: &'static str = "caddy";
    pub(crate) const SHELL_TO_EXEC: &'static str = "bash";
    pub(crate) const DOWNLOAD_CADDY_SCRIPT: &'static str = "download_caddy.bash";
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bin/download_caddy.bash");

    println!("cargo:rerun-if-changed=bin/{}", config::CADDY_BIN_FILE);

    let stdout_file_path = Path::new("build.log");
    if stdout_file_path.is_file() { remove_file(stdout_file_path).unwrap(); }

    let stderr_file_path = Path::new("build.err");
    if stderr_file_path.is_file() { remove_file(stderr_file_path).unwrap(); }

    let shell = which::which(config::SHELL_TO_EXEC).unwrap();

    let mut command = Command::new(shell);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(File::create(stdout_file_path).unwrap()))
        .stderr(Stdio::from(File::create(stderr_file_path).unwrap()))
        .arg(format!("bin/{}", config::DOWNLOAD_CADDY_SCRIPT))
    ;

    match command.output() {
        Ok(_) => (),
        Err(e) => {
            panic!("Could not download Caddy: {}", e);
        }
    };
}