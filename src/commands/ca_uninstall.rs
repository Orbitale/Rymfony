use std::fs::read_to_string;
use std::path::PathBuf;

use openssl::x509::X509;

use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use runas::Command as SudoCommand;

use crate::config::certificates;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("server:ca:uninstall")
        .name("server:ca:uninstall")
        .alias("ca:uninstall")
        .about("Uninstall the local Certificate Authority")
}

pub(crate) fn ca_uninstall(_args: &ArgMatches) {
    let (certificate_path, _key_path) = certificates::get_ca_cert_path().unwrap();

    if !certificate_path.exists() {
        panic!("Unable to uninstall Certificate Authority on your system : no CA certificate generated.");
    }

    if cfg!(target_os = "windows") {
        window_ca_uninstall(&certificate_path);
    } else if cfg!(target_os = "linux") {
        linux_debian_based_ca_uninstall(&certificate_path);
    } else if cfg!(target_os = "macos") {
        macos_ca_uninstall(&certificate_path);
    } else {
        panic!("Unable to install Certificate Authority on your system.")
    }
}

fn linux_debian_based_ca_uninstall(_certificate_path: &PathBuf) {
    let debian_based_cert_path = PathBuf::from("/usr/local/share/ca-certificates/");

    if !debian_based_cert_path.exists() {
        info!("Could not find Certificate Authority directory on your system.");
        return;
    }

    let dest_path = debian_based_cert_path.join("rymfony_CA_cert.crt");

    let status = SudoCommand::new("rm")
        .arg(&dest_path.to_str().unwrap())
        .status()
        .unwrap();

    trace!("Remove result status {}", status);

    let statusupdate = SudoCommand::new("update-ca-certificates")
        .arg("--fresh")
        .status()
        .unwrap();

    trace!("Update CERT result status {}", statusupdate);
}

fn macos_ca_uninstall(certificate_path: &PathBuf) {
    let status = SudoCommand::new("security")
        .arg("remove-trusted-cert")
        .arg("-d")
        .arg(&certificate_path.to_str().unwrap())
        .gui(false)
        .status()
        .unwrap();

    trace!("Copy result status {}", status);
}

fn window_ca_uninstall(certificate_path: &PathBuf) {

    let content = read_to_string(certificate_path).unwrap();
    let certif = X509::from_pem(content.as_bytes()).unwrap();

    let serial = certif.serial_number().to_bn().unwrap();

    let hex = serial.to_hex_str().unwrap();
    trace!("Attempt to remove {}", hex.to_string());

    let status = SudoCommand::new("certutil")
        .arg("-delstore")
        .arg("ROOT")
        .arg(hex.to_string())
        .gui(true)
        .status()
        .unwrap();

    trace!("Copy result status {}", status);
}
