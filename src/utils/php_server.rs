use crate::utils::php_binaries;
use crate::utils::php_server_fpm::start_fpm;

// Check: https://www.php.net/manual/en/install.fpm.configuration.php

pub(crate) fn start() {
    let php_bin = php_binaries::current();

    if php_bin.contains("-fpm") && cfg!(not(target_family = "windows")) {
        start_fpm(php_bin);
    } else if php_bin.contains("-cgi") {
        start_cgi(php_bin);
    } else {
        start_native(php_bin);
    }
}

fn start_cgi(php_bin: String) {
    println!("Todo: serve php-cgi");
}

fn start_native(php_bin: String) {
    println!("Todo: serve php native");
}
