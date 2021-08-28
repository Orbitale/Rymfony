use crate::utils::project_directory::get_rymfony_project_directory;

use regex::Regex;
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use std::process::Stdio;

const CADDY_VERSION_REGEX: &'static str = r"^v(2\.\d+\.\d+) ";

#[cfg(target_os="windows")]
const CADDY_BIN_FILE: &'static str = "caddy.exe";

#[cfg(not(target_os="windows"))]
const CADDY_BIN_FILE: &'static str = "caddy";

pub(crate) const CADDYFILE: &'static str = "
127.0.0.1:{{ http_port }} {
    encode zstd gzip

    @redir_matcher {
        expression {scheme} == 'http'
    }

    redir @redir_matcher https://127.0.0.1:{{ http_port }}{uri}

    {{ use_tls }}
    {{ add_server_sign }}

    root * {{ document_root }}

    php_fastcgi 127.0.0.1:{{ php_port }} {
        index {{ php_entrypoint_file }}
        resolve_root_symlink
    }

    file_server browse
}
";

pub(crate) fn get_caddy_pid_path() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join(".running_caddy.pid")
}

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
                #[cfg(target_os="windows")]
                fs::write(&path, include_bytes!("../../bin/caddy.exe")).expect("Could not extract built-in Caddy binary.");
                #[cfg(not(target_os="windows"))]
                fs::write(&path, include_bytes!("../../bin/caddy")).expect("Could not extract built-in Caddy binary.");
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
        .expect("Could not execute Caddy")
    ;

    let stdout = String::from_utf8(output.stdout).expect("Could not convert Caddy's output to a string.");

    let caddy_version_regex = Regex::new(CADDY_VERSION_REGEX).unwrap();

    if !caddy_version_regex.is_match(&stdout) {
        panic!("Invalid Caddy version output from binary at path \"{}\".", caddy_path.to_str().unwrap())
    }
}