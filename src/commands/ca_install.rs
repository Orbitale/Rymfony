use std::fs::copy;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use dirs;
use runas::Command as SudoCommand;

use crate::config::certificates;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("server:ca:install")
        .name("server:ca:install")
        .about("Create and install a local Certificate Authority for serving HTTPS")
}

pub(crate) fn ca_install(args: &ArgMatches) {
    let (certificate_path, key_path) = certificates::get_ca_cert_path().unwrap();

    if !certificate_path.exists() {
        certificates::get_cert_path().unwrap();
    }
    if !certificate_path.exists() {
        panic!("Unable to generate Certificate Authority at : {}", &certificate_path.to_str().unwrap());
    }

    if cfg!(target_os = "windows") {
        window_ca_install(&certificate_path);
    } else if cfg!(target_os = "linux") {
        ubuntu_ca_install(&certificate_path);
    } else if cfg!(target_os = "macos") {
        macos_ca_install(&certificate_path);
    } else {
        panic!("Unable to install on your system")
    }


    // writeln!("End Of Work");
}

fn ubuntu_ca_install(certificate_path: &PathBuf) {
    let ubuntu_cert_path = PathBuf::from("/usr/local/share/ca-certificates/");

    if !ubuntu_cert_path.exists() {
        info!("Not Ubuntu");
        return;
    }

    let dest_path = ubuntu_cert_path.join("rymfony_CA_cert.crt");

    // copy(&certificates_path.to_str().unwrap(), &dest_path.to_str().unwrap()).expect(&format!("Unable to write on destination file {}", &dest_path.to_str().unwrap()));

    let status = SudoCommand::new("cp")
        .arg(&certificate_path.to_str().unwrap())
        .arg(&dest_path.to_str().unwrap())
        .status()
        .unwrap();

    info!("Copy result status {}", status);

    let statusupdate = SudoCommand::new("update-ca-certificates")
        .status()
        .unwrap();

    info!("Update CERT result status {}", statusupdate);
}

fn macos_ca_install(certificate_path: &PathBuf) {
    let status = SudoCommand::new("security")
        .arg("add-trusted-cert")
        .arg("-d")
        .arg("-r")
        .arg("trustRoot")
        .arg("-k")
        .arg("/Library/Keychains/System.keychain")
        .arg(&certificate_path.to_str().unwrap())
        .gui(false)
        .status()
        .unwrap();

    info!("Copy result status {}", status);
}

fn window_ca_install(certificate_path: &PathBuf) {
    let status = SudoCommand::new("certutil")
        .arg("-addstore")
        .arg("-f")
        .arg("ROOT")
        .arg(&certificate_path.to_str().unwrap())
        .gui(true)
        .status()
        .unwrap();

    info!("Copy result status {}", status);
}
