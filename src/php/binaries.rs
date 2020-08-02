use glob::glob;
use rayon::prelude::*;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::str;
use crate::php::structs::PhpServerSapi;
use crate::php::structs::PhpVersion;
use crate::php::structs::PhpBinary;
use std::collections::HashMap;

pub(crate) fn current() -> String {
    let binaries = all();

    let binaries_regex = if cfg!(target_family = "windows") {
        // On Windows, we mostly have "php" and "php-cgi"
        Regex::new(r"php(\d+(\.\d+))?(-cgi)\.exe$").unwrap()
    } else {
        // This will probably need to be updated for other platforms.
        // This matches "php", "php7.4", "php-fpm", "php7.4-fpm" and "php-fpm7.4"
        Regex::new(r"php(\d+(\.\d+))?(-fpm)(\d+(\.\d+))?$").unwrap()
    };

    for binary in binaries {
        if !binaries_regex.is_match(binary.as_str()) {
            continue;
        }

        // TODO: check for a better solution to choose current PHP version
        return binary.to_string();
    }

    "php".to_string()
}

pub(crate) fn all() -> HashMap<PhpVersion, PhpBinary> {

    let mut binaries: Vec<String> = Vec::new();

    if cfg!(target_family = "windows") {
        binaries.extend(binaries_from_env());
    } else {
        binaries.extend(binaries_from_path(PathBuf::from("/usr/bin")))
    };

    let php_version_output_regex = Regex::new(r"^PHP (\d\.\d+\.\d+) \(([^\)])+\)").unwrap();

    let binaries: Vec<PhpBinary> = binaries
        .into_par_iter()
        .filter(|binary| {
            let version = get_version_from_binary(&binary);
            php_version_output_regex.is_match(version.trim())
        })
        .map(|binary| {
            let version = get_version_from_binary(&binary);

            let capts = php_version_output_regex.captures(&binary).unwrap();

            let sapi = match &capts[2] {
                #[cfg(not(target_os = "windows"))]
                "FPM" => PhpServerSapi::FPM,
                "CLI" => PhpServerSapi::CLI,
                "CGI" => PhpServerSapi::CGI,
                _ => PhpServerSapi::Unknown,
            };

            PhpBinary::from(version.trim().to_string(), PathBuf::from(binary), sapi)
        })
        .collect();

    let mut versions: HashMap<PhpVersion, PhpBinary> = HashMap::new();

    for bin in binaries {
        let php_version = PhpVersion::from_version(&String::from(""));
    }

    versions
}

fn binaries_from_env() -> Vec<String> {
    let path_string = env::var_os("PATH").unwrap();
    let path_dirs = path_string
        .to_str()
        .unwrap()
        .split(if cfg!(target_family = "windows") {
            ";"
        } else {
            ":"
        })
        .collect::<Vec<&str>>();

    path_dirs
        .into_par_iter()
        .map(|dir| {
            binaries_from_path(PathBuf::from(dir))
        })
        .flatten()
        .collect()
}

fn binaries_from_path(path: PathBuf) -> Vec<String> {
    let mut binaries: Vec<String> = vec![];

    if !path.is_dir() {
        println!("not a dir {}", path.to_str().unwrap());

        return binaries;
    }

    let binaries_regex = if cfg!(target_family = "windows") {
        // On Windows, we mostly have "php" and "php-cgi"
        Regex::new(r"php(\d+(\.\d+))?(-cgi)?\.exe$").unwrap()
    } else {
        // This will probably need to be updated for other platforms.
        // This matches "php", "php7.4", "php-fpm", "php7.4-fpm" and "php-fpm7.4"
        Regex::new(r"php(\d+(\.\d+))?(-fpm)?(\d+(\.\d+))?$").unwrap()
    };

    let path_glob = glob_from_path(path.display().to_string().as_str());

    for entry in glob(&path_glob).expect("Failed to read glob pattern") {
        let binary: PathBuf = entry.unwrap();
        if !binaries_regex.is_match(binary.to_str().unwrap()) {
            continue;
        }

        binaries.push(binary.to_str().unwrap().parse().unwrap());
    }

    binaries
}

fn glob_from_path(path: &str) -> String {
    if cfg!(target_family = "windows") {
        format!("{}/php*.exe", path)
    } else {
        format!("{}/php*", path)
    }
}

fn get_version_from_binary(binary: &str) -> String {
    let process = Command::new(binary.as_str())
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = process.wait_with_output().unwrap();

    String::from_utf8(output.stdout).unwrap()
}
