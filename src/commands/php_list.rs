use clap::App;
use clap::SubCommand;

use crate::utils::list_php_binaries;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("php:list").about("List all available PHP executables")
}

pub(crate) fn php_list() {
    let binaries = list_php_binaries::all();

    for binary in binaries {
        println!(" > {}", binary);
    }
}
