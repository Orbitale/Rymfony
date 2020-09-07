use crate::php::binaries;
use crate::php::server_cgi::start as start_cgi;
use crate::php::server_fpm::start as start_fpm;
use crate::php::server_native::start as start_native;
use crate::php::structs::PhpServerSapi;
#[cfg(not(target_os = "windows"))]
use crate::utils::stop_process;

use std::process;
use std::thread;
use std::time;

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
    let php_bin = binaries::current();

    let (php_server, mut process) =
        if php_bin.contains("-fpm") && cfg!(not(target_family = "windows")) {
            start_fpm(php_bin)
        } else if php_bin.contains("-cgi") {
            start_cgi(php_bin)
        } else {
            start_native(php_bin)
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

    ctrlc::set_handler_mut(move || {
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

    php_server
}
