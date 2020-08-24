use std::collections::HashMap;
use crate::php::structs::PhpVersion;
use crate::php::structs::PhpBinary;

pub(crate) fn save_binaries_to_config(binaries: &HashMap<PhpVersion, PhpBinary>) {
    dbg!(&binaries);
}
