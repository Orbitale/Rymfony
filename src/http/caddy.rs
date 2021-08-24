use crate::utils::project_directory::get_rymfony_project_directory;

use sha2::Digest;
use sha2::Sha512;
use std::path::PathBuf;
use std::fs;

#[cfg(target_os="windows")]
const CADDY_BIN_FILE: &'static str = "caddy.exe";

#[cfg(not(target_os="windows"))]
const CADDY_BIN_FILE: &'static str = "caddy";

pub(crate) fn check_caddy_hash() {
    let caddy_path = get_caddy_path();

    if !caddy_path.exists() {
        fs::write(&caddy_path, include_bytes!("../../bin/caddy")).unwrap();
    }

    let caddy_file_bytes = fs::read(caddy_path).unwrap();

    let digest = format!("{:x}", Sha512::digest(&caddy_file_bytes));

    let caddy_checksum = include_str!("../../caddy_checksum.txt").trim().replace("\n", "");

    if digest != caddy_checksum {
        panic!("Caddy checksum is not the same as the one built-in.")
    }
}

fn get_caddy_path() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(CADDY_BIN_FILE)
}