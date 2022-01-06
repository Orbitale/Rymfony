use std::process::Command;
use std::process::Stdio;
use crate::http::caddy::get_caddy_path;
use crate::http::caddy::CADDYFILE;
use crate::config::paths::get_caddy_pid_file;
use crate::utils::project_directory::get_rymfony_project_directory;
use std::path::PathBuf;
use std::fs::File;
use std::fs::read_to_string;
use std::fs;
use std::io::Write;

pub(crate) fn start(
    use_tls: bool,
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

    let caddy_path = get_caddy_path();
    let mut caddy_command = Command::new(&caddy_path);

    // TODO: implement ".wip" (or other) custom domains.
    let host_name = "127.0.0.1".to_string();

    let caddy_config_file = get_caddy_config_file();

    caddy_command
        .stdin(Stdio::piped())
        .stderr(Stdio::from(File::open(http_error_file).expect("Could not open HTTP error file."))) // TODO: check if is this working
        .arg("run")
        .arg("--adapter").arg("caddyfile")
        .arg("--pidfile").arg(get_caddy_pid_file().to_str().unwrap())
        .arg("--config").arg("-") // This makes Caddy use STDIN for config
    ;

    let mut caddy_command = caddy_command
        .spawn()
        .expect("Could not start HTTP server.")
    ;

    let debug = false; // FIXME: use env var for this.

    {
        let mut config: String = if !caddy_config_file.exists() {
            fs::write(&caddy_config_file, CADDYFILE).expect("Could not write Caddyfile config.");
            debug!("Wrote Caddy config to {}", &caddy_config_file.to_str().unwrap());

            CADDYFILE.to_string()
        } else {
            debug!("Reusing Caddy config from {}", &caddy_config_file.to_str().unwrap());

            read_to_string(&caddy_config_file).expect("Could not read Caddyfile config file.")
        };

        config = config
            .replace("{{ document_root }}", document_root.as_str())
            .replace("{{ php_port }}", &php_port.to_string())
            .replace("{{ http_port }}", &http_port.to_string())
            .replace("{{ https_port }}", &http_port.to_string())
            .replace("{{ php_entrypoint_file }}", php_entrypoint_file.as_str())
            .replace("{{ log_file }}", http_log_file.to_str().unwrap())
            .replace("{{ host }}", &host_name)
            .replace("{{ with_server_sign }}", if add_server_sign { "" } else { "#" })
            .replace("{{ without_server_sign }}", if add_server_sign { "#" } else { "" })
            .replace("{{ use_tls }}", if use_tls { "" } else { "#" })
            .replace("{{ debug }}", if debug { "" } else { "#" })
        ;

        trace!("Final Caddy config:\n{}\n", &config);

        let caddy_stdin = caddy_command.stdin.as_mut().unwrap();
        caddy_stdin.write_all(config.as_bytes()).expect("Could not write server config to Caddy STDIN.");
    }

    info!("Listening to {}://{}:{}", if use_tls { "https" } else { "http" }, host_name, http_port);

    let output = caddy_command.wait_with_output().expect("Could not wait for Caddy to finish executing.");

    if output.status.code().unwrap() != 0 {
        let stderr = String::from_utf8(output.stderr).unwrap();

        if stderr.contains("listen tcp :80: bind: permission denied") {
            error!("Caddy is unable to listen to port 80, which is used for HTTP to HTTPS redirection.");
            error!("This can happen when you run Caddy (and therefore Rymfony) as non-root user.");
            error!("To make it work, you need to give Caddy the necessary network capabilities.");

            #[cfg(target_os = "linux")] {
                error!("On most linux distributions, you can do it by running this command (possibly with \"sudo\"):");
                error!("   setcap cap_net_bind_service=+ep {}", caddy_path.to_str().unwrap());
            }
        }

        panic!("Caddy failed to start with error:\n{}", stderr);
    }
}

fn get_http_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("log")
        .join("http.log")
}

fn get_http_error_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("log")
        .join("http.err")
}

fn get_caddy_config_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("Caddyfile")
}
