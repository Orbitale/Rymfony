use crate::utils::project_directory::get_rymfony_project_directory;

use regex::Regex;
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use std::process::Stdio;
#[cfg(not(target_os="windows"))]
use std::os::unix::fs::PermissionsExt;
use atty::Stream;

#[cfg(not(target_os="windows"))]
use runas::Command as SudoCommand;

const CADDY_VERSION_REGEX: &'static str = r"^v(2\.\d+\.\d+) ";

#[cfg(target_os="windows")]
const CADDY_BIN_FILE: &'static str = "caddy.exe";

#[cfg(not(target_os="windows"))]
const CADDY_BIN_FILE: &'static str = "caddy";

pub(crate) const CADDYFILE: &'static str = "
# ⚠⚠⚠
# This file is a *template* created by Rymfony.
# The variables you see in brackets \"{{ … }}\" are
# replaced **at runtime** by Rymfony.
#
# Be warned that if you want to change it,
# it may have an impact on how your local project behaves.
#
# Change it at your own risk 💣

{
    {{ debug }}debug
    log {
        output file {{ log_file }}
        {{ debug }}level DEBUG
    }
    {{ use_tls }}local_certs
    {{ use_tls }}auto_https disable_redirects
}

{{ host }}:{{ http_port }} {
    root * {{ document_root }}

    encode gzip

    {{ with_server_sign }}header Server \"Rymfony\"
    {{ without_server_sign }}header -Server

    log {
        output file {{ vhost_log_file }}
        {{ debug }}level DEBUG
    }

    php_fastcgi 127.0.0.1:{{ php_port }} {
        env SERVER_SOFTWARE \"Rymfony/Caddy\"
        index {{ php_entrypoint_file }}
        resolve_root_symlink
    }

    file_server
}
";

pub(crate) fn get_caddy_path() -> PathBuf {
    let caddy_from_path_env = caddy_from_path_env();

    let caddy_path = match caddy_from_path_env {
        Ok(path) => path,
        Err(_) => {
            let path = get_rymfony_project_directory()
                .expect("Could not get Caddy path from Rymfony directory")
                .join(CADDY_BIN_FILE)
            ;

            if !path.exists() {
                info!("Installing Caddy HTTP server for your project...");

                #[cfg(target_os="windows")]
                fs::write(&path, include_bytes!("../../bin/caddy.exe")).expect("Could not extract built-in Caddy binary.");
                #[cfg(not(target_os="windows"))]
                fs::write(&path, include_bytes!("../../bin/caddy")).expect("Could not extract built-in Caddy binary.");

                #[cfg(not(target_os="windows"))]
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("Could not make Caddy binary executable.");

                // On linux, we try to use "setcap" to give Caddy the ability to listen to port 80
                #[cfg(target_os="linux")]
                set_http_capabilities(&path);
            }

            path
        }
    };

    check_caddy_version(&caddy_path);

    caddy_path
}

fn caddy_from_path_env() -> which::Result<PathBuf> {
    return which::which("caddy");
}

fn check_caddy_version(caddy_path: &PathBuf) {
    let mut command = Command::new(caddy_path);

    let output = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .arg("version")
        .output()
        .expect(&format!("Could not execute Caddy at path \"{}\"", caddy_path.to_str().unwrap()))
    ;

    let stdout = String::from_utf8(output.stdout).expect("Could not convert Caddy's output to a string.");

    let caddy_version_regex = Regex::new(CADDY_VERSION_REGEX).unwrap();

    if !caddy_version_regex.is_match(&stdout) {
        panic!("Invalid Caddy version output from binary at path \"{}\".", caddy_path.to_str().unwrap())
    }
}

#[cfg(target_os="linux")]
fn set_http_capabilities(caddy_path: &PathBuf) {
    warn!("Caddy is usually unable to listen to port 80 when running as non-root user.");
    warn!("This is due a security measure from your OS to not accept non-root executables");
    warn!("to bind ports below 1000.");

    if atty::is(Stream::Stdout) {
        warn!("To make it work, we will try to use the \"setcap\" command,");
        warn!("in order to give Caddy the necessary permissions to listen to port 80.");

        let status = SudoCommand::new("setcap")
            .arg("cap_net_bind_service=+ep")
            .arg(&caddy_path.to_str().unwrap())
            .status()
            .expect("The \"setcap\" command did not execute when trying to give Caddy the ability to listen to port 80.");

        if status.code().unwrap_or(1) != 0 {
            error!("The \"setcap\" command failed when trying to give Caddy the ability to listen to port 80.")
        } else {
            info!("Done! Caddy HTTP server is now capable of listening to port 80 (for this project only)");
        }
    } else {
        warn!("To make it work, you must stop Rymfony and execute this command (as a privileged user):");
        warn!("setcap cap_net_bind_service=+ep {}", &caddy_path.to_str().unwrap());
    }
}
