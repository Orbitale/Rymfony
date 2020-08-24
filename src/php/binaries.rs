use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::str;

use glob::glob;
use regex::Regex;

use crate::php::structs::PhpBinary;
use crate::php::structs::PhpServerSapi;
use crate::php::structs::PhpVersion;

pub(crate) fn current() -> String {
    let binaries = all();

    for (version, binary) in binaries {
        // TODO: check for a better solution to choose current PHP version
        //return binary.path().to_string();
    }

    "php".to_string()
}

pub(crate) fn all() -> HashMap<PhpVersion, PhpBinary> {
    let mut binaries: HashMap<PhpVersion, PhpBinary> = HashMap::new();

    binaries_from_env(&mut binaries);

    merge_binaries(
        binaries_from_dir(PathBuf::from("/usr/bin")),
        &mut binaries
    );

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
        let binaries_from_dir = binaries_from_dir(PathBuf::from(dir));

        for (version, binary) in binaries_from_dir {
            if binaries.contains_key(&version) {
                let mut bin = binaries.get_mut(&version).unwrap();
                bin.merge_with(binary);
            } else {
                binaries.insert(version, binary);
            };
        }
    }
}

fn binaries_from_dir(path: PathBuf) -> HashMap<PhpVersion, PhpBinary> {
    if !path.is_dir() {
        return HashMap::new();
    }

    let binaries_regex = if cfg!(target_family = "windows") {
        // On Windows, we mostly have "php" and "php-cgi"
        Regex::new(r"php(\d+(\.\d+))?(-cgi)?\.exe$").unwrap()
    } else {
        // This will probably need to be updated for other platforms.
        // This matches "php", "php7.4", "php-fpm", "php7.4-fpm" and "php-fpm7.4"
        Regex::new(r"php(\d+(\.\d+))?([_-]?fpm)?(\d+(\.\d+))?$").unwrap()
    };

    let mut binaries_paths: Vec<String> = Vec::new();

    let path_glob = glob_from_path(path.display().to_string().as_str());

    for entry in glob(&path_glob).expect("Failed to read glob pattern") {
        let binary: PathBuf = entry.unwrap();
        if !binaries_regex.is_match(binary.to_str().unwrap()) {
            continue;
        }

        // Canonicalize on Windows leaves the "\\?" prefix on canonicalized paths.
        // Let's not use it, they should be absolute anyway on Windows, so they're usable.
        #[cfg(not(target_family = "windows"))]
        let binary: PathBuf = binary.canonicalize().unwrap();

        binaries_paths.push(binary.to_str().unwrap().parse().unwrap());
    }

    let binaries_paths: HashSet<String> = binaries_paths.iter().cloned().collect();

    let mut binaries: HashMap<PhpVersion, PhpBinary> = HashMap::new();

    for path in binaries_paths.iter() {
        let (version, sapi) = get_binary_metadata(&path);

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

fn glob_from_path(path: &str) -> String {
    if cfg!(target_family = "windows") {
        format!("{}/php*.exe", path)
    } else {
        format!("{}/php*", path)
    }
}

fn get_binary_metadata(binary: &str) -> (PhpVersion, PhpServerSapi) {
    let process = Command::new(binary)
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = process.wait_with_output().unwrap();
    let stdout = output.stdout;
    let output = String::from_utf8(stdout).unwrap();

    let php_version_output_regex = Regex::new(r"^PHP (\d\.\d+\.\d+) \(([^)]+)\)").unwrap();

    if !php_version_output_regex.is_match(&output) {
        panic!(
            "Version \"{}\" for php binary \"{}\" is invalid.",
            &output, &binary
        );
    }

    let capts = php_version_output_regex.captures(&output).unwrap();
    let version = &capts[1];
    let sapi = &capts[2];

    (
        PhpVersion::from_str(version),
        PhpServerSapi::from_str(&sapi),
    )
}

fn merge_binaries(
    from: HashMap<PhpVersion, PhpBinary>,
    into: &mut HashMap<PhpVersion, PhpBinary>
) {
    for (version, binary) in from {
        if into.contains_key(&version) {
            into.get_mut(&version).unwrap().merge_with(binary)
        } else {
            into.insert(version.clone(), PhpBinary::from_version(version.clone()));
        }
    }
}
