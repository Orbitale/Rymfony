use crate::utils::php_binaries;
use crate::utils::php_server_cgi::start as start_cgi;
use crate::utils::php_server_fpm::start as start_fpm;
use crate::utils::php_server_native::start as start_native;
use crate::utils::stop_process;

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

    match process.try_wait() {
        Ok(Some(status)) => panic!(format!("PHP server exited with {}", status)),
        Ok(None) => {
            println!("PHP server seems ready");
        }
        Err(e) => panic!(format!(
            "An error occured when checking PHP server health: {:?}",
            e
        )),
    }

    let pid = process.id();

    ctrlc::set_handler(move || {
        println!("Stopping PHP process...");
        // TODO: I'd like this to work but there are "borrows" issue again -_-
        //process.kill();
        stop_process::stop(pid.to_string().as_ref());
    }).expect("Error setting Ctrl-C handler");
}
