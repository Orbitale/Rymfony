use std::fs::File;
use std::io::Write;
use std::process;
use std::thread;
use std::time;

use crate::php::binaries;
use crate::php::server_cgi::start as start_cgi;
use crate::php::server_fpm::start as start_fpm;
use crate::php::server_native::start as start_native;
use crate::php::structs::{PhpServerSapi, ProcessInfo};
use crate::utils::project_folder::get_rymfony_project_directory;
#[cfg(not(target_os = "windows"))]
use crate::utils::stop_process;
use std::str::FromStr;

pub(crate) struct PhpServer {
    port: u16,
    sapi: PhpServerSapi,
}

impl PhpServer {
    pub(crate) fn new(port: u16, sapi: PhpServerSapi) -> PhpServer {
        PhpServer { port, sapi }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn sapi(&self) -> &PhpServerSapi {
        &self.sapi
    }
}

pub(crate) fn start() -> PhpServer {
    let php_bin = binaries::get_project_version();

    let (php_server, mut process) =
        if php_bin.contains("-fpm") && cfg!(not(target_family = "windows")) {
            start_fpm(php_bin.clone())
        } else if php_bin.contains("-cgi") {
            start_cgi(php_bin.clone())
        } else {
            start_native(php_bin.clone())
        };

    let sleep_time = time::Duration::from_millis(1000);
    thread::sleep(sleep_time);

    let process_status = process.try_wait();

    match process_status {
        Ok(Some(status)) => panic!(format!("PHP server exited with {}", status)),
        Ok(None) => {
            info!("PHP server is ready");
        }
        Err(e) => panic!(format!(
            "An error occured when checking PHP server health: {:?}",
            e
        )),
    }

    let process_pid = process.id();

    ctrlc::set_handler(move || {
        info!("Stopping PHP process... ");

        #[cfg(not(target_os = "windows"))]
        {
            let pid = process.id();
            stop_process::stop(pid.to_string().as_ref()); // Stop fpm children
        }

        match process.kill() {
            Ok(_) => info!("PHP process stopped."),
            Err(e) => info!("An error occured when stopping PHP: {:?}", e),
        }
        process::exit(0);
    })
        .expect("Error setting Ctrl-C handler");

    let args_str: Vec<String> = Vec::new();
    let pidstr = if process_pid > 0 {
        process_pid.to_string()
    } else { "0".to_string() };

    let pid_info = ProcessInfo::new(i32::from_str(pidstr.as_str()).unwrap(), php_server.port, "".to_string(), format!("{}", php_server.sapi), php_bin.clone(), args_str);

    //Serialize
    let serialized = serde_json::to_string_pretty(&pid_info).unwrap();
    let path = get_rymfony_project_directory().unwrap();
    let server_pid_file = path.join("server.pid");
    let mut versions_file = File::create(&server_pid_file).unwrap();

    versions_file.write_all(serialized.as_bytes())
        .expect("Could not write Process informations to JSON file.");


    php_server
}

pub(crate) fn healthcheck(port: u16) -> u16 {
    println!("Checking port {}", &port);

    0
}
