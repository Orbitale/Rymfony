use crate::utils::php_binaries;
use crate::utils::php_server_cgi::start as start_cgi;
use crate::utils::php_server_fpm::start as start_fpm;
use crate::utils::php_server_native::start as start_native;
#[cfg(not(target_os = "windows"))]
use crate::utils::stop_process;

use std::process;
use std::thread;
use std::time;

pub(crate) fn start() {
    let php_bin = php_binaries::current();

    let mut process = if php_bin.contains("-fpm") && cfg!(not(target_family = "windows")) {
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
            println!("PHP server is ready");
        }
        Err(e) => panic!(format!(
            "An error occured when checking PHP server health: {:?}",
            e
        )),
    }

    ctrlc::set_handler_mut(move || {
        println!("Stopping PHP process... ");

        #[cfg(not(target_os = "windows"))]
        {
            let pid = process.id();
            stop_process::stop(pid.to_string().as_ref()); // Stop fpm children
        }

        match process.kill() {
            Ok(_) => println!("PHP process stopped."),
            Err(e) => println!("An error occured when stopping PHP: {:?}", e),
        }
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}
