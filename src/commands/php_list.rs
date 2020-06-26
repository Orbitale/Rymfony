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
        .split(get_path_separator())
        .collect::<Vec<&str>>()
    ;

    let binaries: Vec<String> = path_dirs
         // consumes `path_dirs` and produce an iterator
        .into_iter()
        .map(|dir| {
            // use `PathBuf` directly instead of `Path::new(…).into_owned()`
            binaries_from_path(PathBuf::from(dir))
        })
        // at that point, `binaries_from_path` returns a `Vec<String>` so the iterator is of kind “`Vec<Vec<String>>`”, we only want `Vec<String>`
        .flatten()
        // enjoy!
        .collect();

    let php_version_output_regex = Regex::new(r"").unwrap();

    // you can use the same variable name, it's OK,
    // the previous one will be dropped
    let binaries: Vec<String> = binaries
        // consume `binaries` and generate an iterator
        .into_iter()
        // with `php_version_output_regex`, basically, we want to filter the
        // results, so let's use `Iterator::filter`!
        .filter(|binary| {
            // `Command::new` can take a reference to a string. `binary` is
            // of kind `String`, so just share it by giving a reference
            let process = Command::new(binary.as_str())
                .arg("--version")
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let output = process.wait_with_output().unwrap();
            // instead of getting a slice of the `Vec<u8>` from `output.stdout` to
            // then use `str::from_utf8`, let's just move the ownership from `Vec`
            // (which is its value) to `String` (which also owns its value), it's much
            // simpler!
            let output_string = String::from_utf8(output.stdout).unwrap();

            // finally, no need to call `.to_owned()` after `.is_match(…)`.
            // `is_match` returns a boolean, it's going to be our `filter`'s result
            php_version_output_regex.is_match(output_string.as_str())
        })
        .collect();

    for binary in binaries {
        println!(" > {}", binary);
    }
}

fn get_path_separator() -> &'static str {
    if cfg!(target_family = "windows") {
        ";"
    } else {
        ":"
    }
}

fn binaries_from_path(path: PathBuf) -> Vec<String> {
    let mut binaries: Vec<String>  = vec![];

    let binaries_regex = if cfg!(target_family = "windows") {
        Regex::new(r"php(?:\d\.)*(?:-cgi)\.exe$").unwrap()
    } else {
        Regex::new(r"php(?:\d\.)+(?:-fpm)$").unwrap()
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
