use std::process::Command;
use std::process::Stdio;
use crate::http::caddy::get_caddy_path;
use crate::http::caddy::CADDYFILE;
use crate::http::caddy::get_caddy_pid_path;
use crate::utils::project_directory::get_rymfony_project_directory;
use std::path::PathBuf;
use std::fs::{File, write};

pub(crate) fn start(
    use_tls: bool,
    forward_http_to_https: bool,
    http_port: u16,
    php_port: u16,
    document_root: String,
    php_entrypoint_file: String,
    add_server_sign: bool,
) {
    let http_log_file = get_http_log_file();
    if !http_log_file.exists() { File::create(&http_log_file).expect("Could not create HTTP log file."); }

    let http_error_file = get_http_error_file();
    if !http_error_file.exists() { File::create(&http_error_file).expect("Could not create HTTP error file."); }

    let mut caddy_command = Command::new(get_caddy_path());

    let caddy_config_file = get_caddy_config_file();

    caddy_command
        .stdin(Stdio::piped())
        .stdout(Stdio::from(File::open(http_log_file).expect("Could not open HTTP log file.")))
        .stderr(Stdio::from(File::open(http_error_file).expect("Could not open HTTP error file.")))
        .arg("run")
        .arg("--adapter").arg("caddyfile")
        .arg("--config").arg(&caddy_config_file) // This makes Caddy use STDIN for config
        .arg("--pidfile").arg(get_caddy_pid_path().to_str().unwrap())
        .arg("--watch")
    ;

    let mut caddy_command = caddy_command
        .spawn()
        .expect("Could not start HTTP server.")
    ;

    {
        let config = CADDYFILE
            .replace("{{ document_root }}", document_root.as_str())
            .replace("{{ php_port }}", &php_port.to_string())
            .replace("{{ http_port }}", &http_port.to_string())
            .replace("{{ https_port }}", &http_port.to_string())
            .replace("{{ php_entrypoint_file }}", php_entrypoint_file.as_str())
            .replace("{{ add_server_sign }}", if add_server_sign { "header Server \"Rymfony\"" } else { "header -Server" })
            .replace("{{ use_tls }}", if use_tls { "tls internal" } else { "" })
            .replace("{{ forward_http_to_https }}", if forward_http_to_https { "" } else { "auto_https off" })
        ;

        // if !caddy_config_file.exists() {
            write(&caddy_config_file, &config).expect("Could not server config to Caddyfile.");
        // }

        println!("Caddyfile:\n{}\n", &config);

        // let caddy_stdin = caddy_command.stdin.as_mut().unwrap();
        // caddy_stdin.write_all(config.as_bytes()).expect("Could not write server config to Caddy STDIN.");
    }

    let output = caddy_command.wait_with_output().expect("Could not wait for Caddy to finish executing.");

    dbg!(output);
}

fn get_http_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("http.log")
}

fn get_http_error_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("http.err")
}

fn get_caddy_config_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("Caddyfile")
}
