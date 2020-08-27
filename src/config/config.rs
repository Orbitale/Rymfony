use std::collections::HashMap;
use crate::php::structs::PhpVersion;
use crate::php::structs::PhpBinary;
use dirs::home_dir;
use std::fs::File;
use std::io::Write;

pub(crate) fn save_binaries_to_config(binaries: &HashMap<PhpVersion, PhpBinary>) {
    let serialized = serde_json::to_string_pretty(&binaries).unwrap();

    let versions_file_path = home_dir().unwrap().join(".rymfony").join("php-versions.json");

    let mut versions_file = File::create(versions_file_path).unwrap();

    versions_file.write_all(serialized.as_bytes())
        .expect("Could not write PHP versions to JSON file.");
}
