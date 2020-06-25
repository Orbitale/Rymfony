use clap::App;
use clap::SubCommand;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use glob::glob;
use std::process::Command;
use std::process::Stdio;
use regex::Regex;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("php:list").about("List all available PHP executables")
}

pub(crate) fn php_list() {
    let path_string = env::var_os("PATH").unwrap();
    let path_dirs = path_string
        .to_str()
        .unwrap()
        .split(get_path_separator().as_str())
        .collect::<Vec<&str>>()
    ;

    let mut binaries: Vec<String> = vec![];

    for dir in &path_dirs {
        let path = Path::new(dir);
        binaries.append(&mut binaries_from_path(path.to_owned()));
    }

    let php_version_output_regex = Regex::new(r"").unwrap();

    let mut final_binaries: Vec<&str> = vec![];

    for binary in binaries {
        let process = Command::new(binary.to_string())
            .arg("--version")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
        ;

        let output = process.wait_with_output().unwrap();
        let chars = output.stdout.as_slice();
        let output_string = str::from_utf8(chars).unwrap();

        if php_version_output_regex.is_match(output_string).to_owned() {
            let bin_string = binary.to_owned();
            final_binaries.push(bin_string.as_ref());
        }
    };

    for binary in final_binaries {
        println!(" > {}", binary);
    }
}

fn get_path_separator() -> String {
    if cfg!(target_family = "windows") {
        String::from(";")
    } else {
        String::from(":")
    }
}

fn binaries_from_path(path: PathBuf) -> Vec<String> {
    let mut binaries: Vec<String>  = vec![];

    let binaries_regex = match cfg!(target_family = "windows") {
        true => Regex::new(r"php(?:\d\.)*(?:-cgi)\.exe$").unwrap(),
        false => Regex::new(r"php(?:\d\.)+(?:-fpm)$").unwrap()
    };

    let path_glob = glob_from_path(path.display().to_string().as_str());

    for entry in glob(&path_glob).expect("Failed to read glob pattern") {
        let binary: PathBuf = entry.unwrap();
        if !binaries_regex.is_match(binary.to_str().unwrap()) {
            continue
        }

        binaries.push(path.display().to_string());
    }

    binaries
}

fn glob_from_path(path: &str) -> String {
    if cfg!(target_family = "windows") {
        format!("{}/php*.exe", path)
    } else {
        format!("{}/php{{,-fpm}}", path)
    }
}
