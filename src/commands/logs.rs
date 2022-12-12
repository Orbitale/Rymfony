use crate::command_handling::CommandHandler;
use crate::config::paths;
use clap::arg;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use colored::*;
use linemux::MuxedLines;
use std::process::ExitCode;

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(
        ClapCommand::new("logs")
            .alias("log")
            .alias("local:server:log") // For Symfony CLI compat
            .alias("server:log") // For Symfony CLI compat
            .about("Display server logs")
            .arg(arg!(<channel> "The optional logging channel you want to display"))
            .arg(arg!(--"no-follow" "Do not tail the logs").alias("no-tail"))
            .arg(arg!(-n --lines <LINES> "Number of lines to display at start").default_value("0")),
        Box::new(execute),
    )
}

pub(crate) fn execute(_args: &ArgMatches) -> ExitCode {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut lines = MuxedLines::new().unwrap();

        let rymfony_process_log_file = paths::get_rymfony_process_log_file();
        let rymfony_process_err_file = paths::get_rymfony_process_err_file();
        let http_process_stdout_file = paths::get_http_process_stdout_file();
        let http_process_stderr_file = paths::get_http_process_stderr_file();
        let http_server_log_file = paths::get_http_server_log_file();
        let http_vhost_log_file = paths::get_http_vhost_log_file();
        let php_process_log_file = paths::get_php_process_log_file();
        let php_process_err_file = paths::get_php_process_err_file();
        let php_server_log_file = paths::get_php_server_log_file();
        let php_server_error_file = paths::get_php_server_error_file();

        info!("Tailing following channels:");
        info!("  {}", rymfony_process_log_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", rymfony_process_err_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", http_process_stdout_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", http_process_stderr_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", http_server_log_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", http_vhost_log_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", php_process_log_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", php_process_err_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", php_server_log_file.file_name().unwrap().to_str().unwrap());
        info!("  {}", php_server_error_file.file_name().unwrap().to_str().unwrap());

        lines.add_file(rymfony_process_log_file).await.unwrap();
        lines.add_file(rymfony_process_err_file).await.unwrap();

        lines.add_file(http_process_stdout_file).await.unwrap();
        lines.add_file(http_process_stderr_file).await.unwrap();

        lines.add_file(http_server_log_file).await.unwrap();
        lines.add_file(http_vhost_log_file).await.unwrap();

        lines.add_file(php_process_log_file).await.unwrap();
        lines.add_file(php_process_err_file).await.unwrap();

        lines.add_file(php_server_log_file).await.unwrap();
        lines.add_file(php_server_error_file).await.unwrap();

        while let Ok(Some(line)) = lines.next_line().await {
            let source = line.source().file_name().unwrap();
            println!("[{}] - {}", source.to_str().unwrap().green(), line.line());
        }
    });

    ExitCode::from(0)
}
