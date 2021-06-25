use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::str;

use glob::glob;
use glob::GlobError;
use is_executable::is_executable;
use regex::Regex;

use crate::config::config::load_binaries_from_config;
use crate::php::structs::PhpBinary;
use crate::php::structs::PhpServerSapi;
use crate::php::structs::PhpVersion;
use std::ffi::OsString;

pub(crate) fn get_project_version() -> String {
    let _binaries = all();

    let project_php_version_file_path = env::current_dir().unwrap().join(".php-version");

    let mut php_version = if project_php_version_file_path.exists() {
        read_to_string(project_php_version_file_path).unwrap()
    } else {
        String::from("")
    };

    php_version = php_version.trim().to_string();

    if php_version != "" {
        debug!(
            "PHP version set to {} from \".php-version\" file.",
            php_version
        );
    }

    let mut system = String::from("");
    let mut user_selected = String::from("");
    let mut user_selected_version = String::from("");

    for (_version, _binary) in _binaries {
        if php_version != "" && _version.version().starts_with(&php_version) {
            if user_selected_version.eq("") || user_selected_version.as_str() < _version.version() {
                user_selected_version = String::from(_version.version());
                user_selected = _binary.preferred_sapi().to_string();
            }
        }
        if _binary.system() {
            system = _binary.preferred_sapi().to_string();
        }
    }

    if user_selected.ne("") {
        trace!("User selected version {}", user_selected_version);
        return user_selected;
    }
    if system.ne("") {
        trace!("System version selected");
        return system;
    }

    "php".to_string()
}

pub(crate) fn all() -> HashMap<PhpVersion, PhpBinary> {
    let load_infos = load_binaries_from_config();
    return match load_infos {
        Ok(data) => data,
        Err(_) => get_all(),
    };
}

fn get_all() -> HashMap<PhpVersion, PhpBinary> {
    let mut binaries: HashMap<PhpVersion, PhpBinary> = HashMap::new();

    binaries_from_env(&mut binaries);
    binaries_from_rymfony_env(&mut binaries);

    merge_binaries(&mut binaries, binaries_from_dir(PathBuf::from("/usr/bin")));
    merge_binaries(&mut binaries, binaries_from_dir(PathBuf::from("/usr/sbin")));
    merge_binaries(
        &mut binaries,
        binaries_from_dir(PathBuf::from("/usr/local/Cellar/php/*/bin")),
    );
    merge_binaries(
        &mut binaries,
        binaries_from_dir(PathBuf::from("/usr/local/Cellar/php@*/*/bin")),
    );
    merge_binaries(
        &mut binaries,
        binaries_from_dir(PathBuf::from("/usr/local/php*/bin")),
    );

    if cfg!(target_family = "windows") {
        merge_binaries(&mut binaries, binaries_from_dir(PathBuf::from("c:\\php")));
    }

    binaries
}

fn binaries_from_env(binaries: &mut HashMap<PhpVersion, PhpBinary>) {
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

    for dir in path_dirs {
        trace!("Checking path for PHP binaries: {}", &dir);
        merge_binaries(binaries, binaries_from_dir(PathBuf::from(dir)));
    }
}

fn binaries_from_rymfony_env(binaries: &mut HashMap<PhpVersion, PhpBinary>) {
    let path_string = env::var_os("RYMFONY_PATH").unwrap_or(OsString::from(""));
    let path_dirs = path_string
        .to_str()
        .unwrap()
        .split(if cfg!(target_family = "windows") {
            ";"
        } else {
            ":"
        })
        .collect::<Vec<&str>>();

    for dir in path_dirs {
        merge_binaries(binaries, binaries_from_dir(PathBuf::from(dir)));
    }
}

fn binaries_from_dir(path: PathBuf) -> HashMap<PhpVersion, PhpBinary> {
    // This matches executables like "php", "php74", "php7.4", "php-fpm", "php7.4-cgi" or "php-fpm74"
    let binaries_regex = if cfg!(target_family = "windows") {
        // On Windows, we mostly have "php" and "php-cgi"
        Regex::new(r"php(\d+(\.\d+)*)?(-cgi)?\.(exe|bat|cmd)$").unwrap()
    } else {
        // This will probably need to be updated for other platforms.
        Regex::new(r"php(\d+(\.\d+)*)?([_-]?fpm|[_-]?cgi)?(\d+(\.\d+)*)?$").unwrap()
    };

    let mut binaries_paths: Vec<String> = Vec::new();

    let mut path = path.display().to_string();
    if path.ends_with('/') {
        path.pop();
    }
    if path.ends_with('\\') {
        path.pop();
    }

    let entries: Vec<Result<PathBuf, GlobError>> = if cfg!(target_family = "windows") {
        let glob_path_exe = format!("{}/php*.exe", path);
        let glob_path_cmd = format!("{}/php*.cmd", path);
        let glob_path_bat = format!("{}/php*.bat", path);
        glob(&glob_path_exe.as_str())
            .unwrap()
            .chain(glob(&glob_path_cmd.as_str()).unwrap())
            .chain(glob(&glob_path_bat.as_str()).unwrap())
            .collect()
    } else {
        let glob_path = format!("{}/php*", path);
        glob(&glob_path.as_str()).unwrap().collect()
    };

    for entry in entries {
        let binary: PathBuf = entry.unwrap();
        if binary.is_dir() {
            // This means that we have a "php"-like dir.
            // For recursive search, insert a "*" glob character in the "path" variable beforehand.
            trace!(
                "Found path \"{}\", but it is a directory.",
                &binary.to_str().unwrap()
            );
            continue;
        }

        if !binaries_regex.is_match(binary.to_str().unwrap()) {
            trace!("Path \"{}\" does not match.", &binary.to_str().unwrap());
            continue;
        }

        if cfg!(not(target_family = "windows")) && !is_executable(&binary) {
            warn!(
                "Path \"{}\" matches, but it is not executable.",
                &binary.to_str().unwrap()
            );
            continue;
        }

        // Canonicalize on Windows leaves the "\\?" prefix on canonicalized paths, which causes issues.
        // Let's not use it, they should be absolute anyway on Windows, so they're usable.
        #[cfg(not(target_family = "windows"))]
        let binary: PathBuf = binary.canonicalize().unwrap();

        binaries_paths.push(binary.to_str().unwrap().parse().unwrap());
    }

    let binaries_paths: HashSet<String> = binaries_paths.iter().cloned().collect();

    let mut binaries: HashMap<PhpVersion, PhpBinary> = HashMap::new();

    for path in binaries_paths.iter() {
        let binary = get_binary_metadata(&path);

        if binary.is_err() {
            continue;
        }

        let (version, sapi) = binary.unwrap();

        if binaries.contains_key(&version) {
            let current = &mut binaries.get_mut(&version).unwrap();

            if !current.has_sapi(&sapi) {
                current.add_sapi(&sapi, &path);
            }
        } else {
            let mut bin = PhpBinary::from_version(version.clone());
            bin.add_sapi(&sapi, &path);
            &binaries.insert(version.clone(), bin);
        }
    }

    binaries
}

fn get_binary_metadata(binary: &str) -> Result<(PhpVersion, PhpServerSapi), ()> {
    let process_result = Command::new(binary)
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn();

    if process_result.is_err() {
        debug!(
            "Path \"{}\" was detected, but is not a valid PHP binary.",
            &binary
        );

        return Err(());
    }

    let process = process_result.unwrap();

    let output = process.wait_with_output().unwrap();
    let stdout = output.stdout;
    let output = String::from_utf8(stdout).unwrap();

    let php_version_output_regex = Regex::new(r"^PHP (\d\.\d+\.\d+)[^ ]* \(([^)]+)\)").unwrap();

    if !php_version_output_regex.is_match(&output) {
        panic!(
            "Version \"{}\" for PHP binary \"{}\" is invalid.",
            &output, &binary
        );
    }

    let capts = php_version_output_regex.captures(&output).unwrap();
    let version = &capts[1];
    let sapi = &capts[2];

    Ok((
        PhpVersion::from_str(version),
        PhpServerSapi::from_str(&sapi),
    ))
}

fn merge_binaries(into: &mut HashMap<PhpVersion, PhpBinary>, from: HashMap<PhpVersion, PhpBinary>) {
    for (version, mut binary) in from {
        // FIXME
        // this needs to be fixed, but for now we assume that the first ever found version is
        // the one that is first in PATH and therefore the "system" binary
        &binary.set_system(if into.len() == 0 { true } else { false });

        if into.contains_key(&version) {
            into.get_mut(&version).unwrap().merge_with(binary);
        } else {
            into.insert(version, binary);
        }
    }
}
