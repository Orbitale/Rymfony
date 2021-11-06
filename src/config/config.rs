use crate::php::structs::PhpBinary;
use crate::php::structs::PhpVersion;
use dirs::home_dir;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::fs::File;
use std::io::Write;
use crate::config::paths::php_server_pid_file;

#[derive(Debug)]
struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured: {}", self.0)
    }
}

impl Error for ConfigError {}

pub(crate) fn save_binaries_to_config(binaries: &HashMap<PhpVersion, PhpBinary>) {
    let serialized = serde_json::to_string_pretty(&binaries).unwrap();

    let versions_file_path = home_dir()
        .unwrap()
        .join(".rymfony")
        .join("php-versions.json");

    let mut versions_file = File::create(versions_file_path).unwrap();

    versions_file
        .write_all(serialized.as_bytes())
        .expect("Could not write PHP versions to cache file.");
}

pub(crate) fn load_binaries_from_config(
) -> std::result::Result<HashMap<PhpVersion, PhpBinary>, Box<dyn std::error::Error>> {
    let versions_file_path = home_dir()
        .unwrap()
        .join(".rymfony")
        .join("php-versions.json");

    if !versions_file_path.exists() {
        return Err(Box::new(ConfigError("No file found".into())));
    }
    trace!("File {} found", versions_file_path.to_str().unwrap());

    let binaries: HashMap<PhpVersion, PhpBinary> =
        serde_json::from_str(read_to_string(&versions_file_path).unwrap().as_str())
            .expect("Unable to unserialize PHP binaries data");

    Ok(binaries)
}

pub(crate) fn clear_binaries_list() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let versions_file_path = home_dir()
        .unwrap()
        .join(".rymfony")
        .join("php-versions.json");

    if versions_file_path.exists() {
        remove_file(&versions_file_path).expect(
            format!(
                "Unable to remove cache file {}",
                versions_file_path.to_str().unwrap()
            )
            .as_str(),
        );
    }
    Ok(())
}

pub(crate) fn php_server_pid() -> String {
    String::from(read_to_string(php_server_pid_file()).expect("Could not read PHP server's PID file."))
}
