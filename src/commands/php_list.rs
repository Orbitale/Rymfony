use clap::App;
use clap::SubCommand;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use glob::glob;
use std::process::Command;
use std::process::Stdio;

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
        binaries.append(binaries_from_path(path.to_owned()).as_ref());
    }

    for binary in binaries {

        let mut process = Command::new(binary)
            .arg("--version")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap()
            ;

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

    let path_glob = glob_from_path(path.display().to_string().as_str());

    for entry in glob(&path_glob).expect("Failed to read glob pattern") {
        // TODO: regex matching
        match entry {
            Ok(path) => {
                binaries.push(path.display().to_string());
                println!("{}", path.display().to_string());
            },
            Err(e) => println!("{:?}", e),
        }
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
