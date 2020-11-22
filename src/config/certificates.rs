use openssl::rsa::Rsa;
use std::path::PathBuf;
use dirs::home_dir;
use std::fs::File;
use std::io::Write;
use openssl::pkey::PKey;
use openssl::pkey::Private;
use openssl::x509::X509NameBuilder;
use openssl::x509::X509;
use openssl::error::ErrorStack;
use openssl::bn::BigNum;
use openssl::bn::MsbOption;
use openssl::asn1::Asn1Time;
use openssl::x509::extension::KeyUsage;
use openssl::x509::extension::BasicConstraints;
use openssl::x509::extension::SubjectKeyIdentifier;
use openssl::hash::MessageDigest;

type CertResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn get_cert_path() -> CertResult<(PathBuf, PathBuf)>
{
    let certificate_path = home_dir().unwrap().join(".rymfony").join("tls_cert.pem");
    let key_path = home_dir().unwrap().join(".rymfony").join("tls_key.pem");

    if certificate_path.exists() && key_path.exists() {
        return Ok((certificate_path, key_path));
    }

    let mut cert_file = if certificate_path.exists() {
        File::open(&certificate_path)
    } else {
        File::create(&certificate_path)
    }?;

    let mut key_file = if key_path.exists() {
        File::open(&key_path)
    } else {
        File::create(&key_path)
    }?;

    let (certificate, private_key) = generate_key_pair()?;

    cert_file.write_all(&certificate.to_pem()?)?;
    key_file.write_all(&private_key.private_key_to_pem_pkcs8()?)?;

    return Ok((certificate_path, key_path));
}

fn generate_key_pair() -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let privkey = PKey::from_rsa(rsa)?;

    let mut x509_name = X509NameBuilder::new()?;
    x509_name.append_entry_by_text("C", "FR")?;
    x509_name.append_entry_by_text("O", "Orbitale.io")?;
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
            .crl_sign()
            .build()?,
    )?;

    let subject_key_identifier =
        SubjectKeyIdentifier::new().build(&cert_builder.x509v3_context(None, None))?;
    cert_builder.append_extension(subject_key_identifier)?;

    cert_builder.sign(&privkey, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, privkey))
}
