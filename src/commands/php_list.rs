use clap::App;
use clap::SubCommand;
use prettytable::format;
use prettytable::Table;

use crate::php;
use crate::config::config::save_binaries_to_config;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("php:list").about("List all available PHP executables")
}

pub(crate) fn php_list() {
    let binaries = php::binaries::all();

    save_binaries_to_config(&binaries);

    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(
            &[format::LinePosition::Top],
            format::LineSeparator::new('─', '┬', '┌', '┐'),
        )
        .separators(
            &[format::LinePosition::Bottom],
            format::LineSeparator::new('─', '┴', '└', '┘'),
        )
        .separators(
            &[format::LinePosition::Title],
            format::LineSeparator::new('─', '┼', '├', '┤'),
        )
        .padding(1, 1)
        .build();

    table.set_format(format);
    table.set_titles(row!["Version", "PHP CLI", "PHP FPM", "PHP CGI"]);

    let mut ordered_binaries: Vec<_> = binaries.into_iter().collect();
    ordered.sort_by(|x,y| x.0.version().cmp(y.0.version()));

    for (php_version, php_binary) in ordered_binaries {
        table.add_row(row![
            php_version.version(),
            php_binary.cli(),
            php_binary.fpm(),
            php_binary.cgi(),
        ]);
    }

    table.printstd();
}
