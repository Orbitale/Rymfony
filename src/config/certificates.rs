use std::fs::File;
use std::fs::metadata;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;

use dirs::home_dir;
use openssl::asn1::Asn1Time;
use openssl::bn::BigNum;
use openssl::bn::MsbOption;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, PKeyRef};
use openssl::pkey::Private;
use openssl::rsa::Rsa;
use openssl::x509::{X509NameBuilder, X509Ref, X509Req, X509ReqBuilder};
use openssl::x509::extension::{AuthorityKeyIdentifier, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName};
use openssl::x509::extension::BasicConstraints;
use openssl::x509::extension::SubjectKeyIdentifier;
use openssl::x509::X509;

type CertResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn get_cert_path() -> CertResult<(PathBuf, PathBuf)>
{
    let certificate_path = home_dir().unwrap().join(".rymfony").join("tls_cert.pem");
    let key_path = home_dir().unwrap().join(".rymfony").join("tls_key.pem");

    let certificate_is_not_empty = file_is_not_empty(&certificate_path);
    let key_is_not_empty = file_is_not_empty(&key_path);

    if certificate_is_not_empty && key_is_not_empty {
        return Ok((certificate_path, key_path));
    }

    let mut cert_file = if certificate_is_not_empty {
        File::open(&certificate_path)
    } else {
        File::create(&certificate_path)
    }?;

    let mut key_file = if key_is_not_empty {
        File::open(&key_path)
    } else {
        File::create(&key_path)
    }?;

    let (ca_cert, ca_privkey) = load_or_generate_ca_cert()?;

    let (certificate, private_key) = generate_ca_signed_cert(&ca_cert, &ca_privkey)?;

    cert_file.write_all(&certificate.to_pem()?)?;
    cert_file.write(&ca_cert.to_pem()?)?;
    key_file.write_all(&private_key.private_key_to_pem_pkcs8()?)?;

    return Ok((certificate_path, key_path));
}

pub(crate) fn get_ca_cert_path() -> CertResult<(PathBuf, PathBuf)>
{
    let certificate_path = home_dir().unwrap().join(".rymfony").join("ca_tls_cert.pem");
    let key_path = home_dir().unwrap().join(".rymfony").join("ca_tls_key.pem");
    Ok((certificate_path, key_path))
}

fn load_or_generate_ca_cert() -> CertResult<(X509, PKey<Private>)> {
    let (ca_path, ca_key_path) = get_ca_cert_path()?;

    if file_is_not_empty(&ca_path) && file_is_not_empty(&ca_key_path) {
        let (ca_cert, ca_privkey) = load_ca_cert(&ca_path, &ca_key_path)?;
        return Ok((ca_cert, ca_privkey));
    }

    let (ca_cert, ca_privkey) = generate_ca_key_pair()?;

    let mut cert_file = if file_is_not_empty(&ca_path) {
        File::open(&ca_path)
    } else {
        File::create(&ca_path)
    }?;

    let mut key_file = if file_is_not_empty(&ca_key_path) {
        File::open(&ca_key_path)
    } else {
        File::create(&ca_key_path)
    }?;

    cert_file.write_all(&ca_cert.to_pem()?)?;
    key_file.write_all(&ca_privkey.private_key_to_pem_pkcs8()?)?;

    Ok((ca_cert, ca_privkey))
}

fn load_ca_cert(certificate_path: &PathBuf, key_path: &PathBuf) -> Result<(X509, PKey<Private>), ErrorStack> {
    let content = read_to_string(certificate_path).unwrap();
    let certif = X509::from_pem(content.as_bytes()).unwrap();

    let content_key = read_to_string(key_path).unwrap();
    let pkey = PKey::private_key_from_pem(content_key.as_bytes()).unwrap();

    Ok((certif, pkey))
}

fn generate_ca_key_pair() -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let privkey = PKey::from_rsa(rsa)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "FR")?;
    x509_name.append_entry_by_text("O", "Orbitale.io")?;
    x509_name.append_entry_by_text("OU", "Orbitale.io localhost")?;
    x509_name.append_entry_by_text("CN", "Orbitale CA (dev)")?;
    let x509_name = x509_name.build();

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(&x509_name)?;
    cert_builder.set_issuer_name(&x509_name)?;
    cert_builder.set_pubkey(&privkey)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().ca().build()?)?;
    cert_builder.append_extension(
        KeyUsage::new()
            .critical()
            .key_cert_sign()
            .build()?,
    )?;

    let subject_key_identifier =
        SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    cert_builder.sign(&privkey, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, privkey))
}

fn generate_request(privkey: &PKey<Private>) -> Result<X509Req, ErrorStack> {
    let mut req_builder = X509ReqBuilder::new()?;
    req_builder.set_pubkey(&privkey)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "FR")?;
    x509_name.append_entry_by_text("O", "Orbitale.io cert (dev)")?;
    x509_name.append_entry_by_text("OU", "Orbitale localhost (dev)")?;
    let x509_name = x509_name.build();
    req_builder.set_subject_name(&x509_name)?;

    req_builder.sign(&privkey, MessageDigest::sha256())?;
    let req = req_builder.build();
    Ok(req)
}

fn generate_ca_signed_cert(
    ca_cert: &X509Ref,
    ca_privkey: &PKeyRef<Private>,
) -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let privkey = PKey::from_rsa(rsa)?;

    let req = generate_request(&privkey)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    cert_builder.set_serial_number(&serial_number)?;
    cert_builder.set_subject_name(req.subject_name())?;
    cert_builder.set_issuer_name(ca_cert.subject_name())?;
    cert_builder.set_pubkey(&privkey)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.append_extension(BasicConstraints::new().critical().build()?)?;

    cert_builder.append_extension(
        KeyUsage::new()
            .critical()
            .digital_signature()
            .key_encipherment()
            .build()?,
    )?;
    cert_builder.append_extension(
        ExtendedKeyUsage::new().server_auth().build()?,
    )?;

    let subject_key_identifier =
        SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    let auth_key_identifier = AuthorityKeyIdentifier::new()
        .keyid(false)
        .issuer(false)
        .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(auth_key_identifier)?;

    let subject_alt_name = SubjectAlternativeName::new()
        .dns("localhost")
        .ip("127.0.0.1")
        .ip("::1")
        .build(&cert_builder.x509v3_context(Some(ca_cert), None))?;
    cert_builder.append_extension(subject_alt_name)?;

    cert_builder.sign(&ca_privkey, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, privkey))
}

fn file_is_not_empty(path: &PathBuf) -> bool {
    path.exists() && metadata(&path.to_str().unwrap()).unwrap().len() > 0
}
