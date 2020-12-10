use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use regex::Regex;
use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::Deserialize as SerdeDeserialize;
use serde::Serialize;
use serde::Serializer;

#[derive(Debug)]
pub(crate) enum PhpServerSapi {
    FPM,
    CGI,
    CLI,
    Unknown,
}

impl PhpServerSapi {
    pub(crate) fn from_str(str: &str) -> PhpServerSapi {
        let str = str.to_lowercase();
        match str.as_str() {
            "fpm" => PhpServerSapi::FPM,
            "fpm-fcgi" => PhpServerSapi::FPM,
            "cli" => PhpServerSapi::CLI,
            "cgi" => PhpServerSapi::CGI,
            "cgi-fcgi" => PhpServerSapi::CGI,
            _ => PhpServerSapi::Unknown,
        }
    }

    pub(crate) fn all() -> Vec<PhpServerSapi> {
        vec![
            PhpServerSapi::FPM,
            PhpServerSapi::CLI,
            PhpServerSapi::CGI,
        ]
    }
}

impl Display for PhpServerSapi {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self {
            PhpServerSapi::FPM => write!(f, "FPM"),
            PhpServerSapi::CLI => write!(f, "CLI"),
            PhpServerSapi::CGI => write!(f, "CGI"),
            PhpServerSapi::Unknown => write!(f, "unknown"),
        }
    }
}

//
//
//
//
//
//
//
//

#[derive(Hash, Eq, PartialEq, Debug)]
pub(crate) struct PhpVersion {
    _version: String,
}

impl PhpVersion {
    pub(crate) fn clone(&self) -> PhpVersion {
        PhpVersion {
            _version: self._version.clone()
        }
    }

    pub(crate) fn from_string(version: String) -> PhpVersion {
        let version_regex = Regex::new("^[578]\\.\\d{1,2}\\.\\d+$").unwrap();

        if !version_regex.is_match(&version) {
            panic!("Version \"{}\" is not a valid PHP version.", &version);
        }

        PhpVersion {
            _version: version.clone(),
        }
    }

    pub(crate) fn new() -> PhpVersion {
        PhpVersion {
            _version: "".to_string(),
        }
    }

    pub(crate) fn from_str(version: &str) -> PhpVersion {
        PhpVersion::from_string(version.to_string())
    }

    pub(crate) fn version(&self) -> &str {
        self._version.as_str()
    }
}

impl Serialize for PhpVersion {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_str(&self._version.as_str())
    }
}

impl<'de> Deserialize<'de> for PhpVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        deserializer.deserialize_str(PhpVersionVisitor)
    }
}

struct PhpVersionVisitor;

impl<'de> Visitor<'de> for PhpVersionVisitor {
    type Value = PhpVersion;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
    {
        Ok(PhpVersion::from_str(value))
    }
}

//
//
//
//
//
//
//
//


#[derive(Hash, Eq, PartialEq, Debug, Serialize, SerdeDeserialize)]
pub(crate) struct PhpBinary {
    cli: String,
    fpm: String,
    cgi: String,
    system: bool,
    #[serde(skip_serializing)]
    #[serde(default = "PhpVersion::new")]
    _version: PhpVersion,
}

impl PhpBinary {
    pub(crate) fn from_version(version: PhpVersion) -> PhpBinary {
        PhpBinary {
            _version: version,
            cli: String::from(""),
            fpm: String::from(""),
            cgi: String::from(""),
            system: false,
        }
    }

    pub(crate) fn merge_with(&mut self, from: PhpBinary) {
        if self._version != from._version {
            return;
        }

        for sapi in PhpServerSapi::all() {
            if from.has_sapi(&sapi) && !self.has_sapi(&sapi) {
                self.add_sapi(&sapi, &from.sapi_path(&sapi));
            }
        }
    }

    pub(crate) fn cli(&self) -> &String {
        &self.cli
    }
    pub(crate) fn fpm(&self) -> &String {
        &self.fpm
    }
    pub(crate) fn cgi(&self) -> &String {
        &self.cgi
    }

    pub(crate) fn has_sapi(&self, sapi: &PhpServerSapi) -> bool {
        match sapi {
            PhpServerSapi::FPM => self.fpm != "",
            PhpServerSapi::CLI => self.cli != "",
            PhpServerSapi::CGI => self.cgi != "",
            PhpServerSapi::Unknown => false,
        }
    }

    pub(crate) fn system(&self) -> bool {
        self.system.clone()
    }

    pub(crate) fn set_system(&mut self, is_system: bool) {
        self.system = is_system;
    }

    pub(crate) fn preferred_sapi(&self) -> String {
        if self.fpm != "" {
            return self.fpm.clone();
        } else if self.cgi != "" {
            return self.cgi.clone();
        } else if self.cli != "" {
            return self.cli.clone();
        } else {
            panic!("Cannot detect preferred sapi for PHP \"{}\"", self._version.version());
        }
    }

    pub(crate) fn add_sapi(&mut self, sapi: &PhpServerSapi, path: &String) {
        match sapi {
            PhpServerSapi::FPM => {
                self.fpm = path.clone();
            }
            PhpServerSapi::CLI => {
                self.cli = path.clone();
            }
            PhpServerSapi::CGI => {
                self.cgi = path.clone();
            }
            PhpServerSapi::Unknown => {
                panic!("Unknown sapi \"{}\" at path \"{}\"", &sapi, &path);
            }
        }
    }

    pub(crate) fn sapi_path(&self, sapi: &PhpServerSapi) -> String {
        match sapi {
            PhpServerSapi::FPM => self.fpm.clone(),
            PhpServerSapi::CLI => self.cli.clone(),
            PhpServerSapi::CGI => self.cgi.clone(),
            PhpServerSapi::Unknown => String::from(""),
        }
    }
}


//
//
//
//
//
//
//
//


#[derive(Hash, Eq, PartialEq, Debug, Serialize, SerdeDeserialize)]
pub(crate) struct ServerInfo {
    pid: i32,
    port: u16,
    scheme: String,
    name: String,
    command: String,
    args: Vec<String>,
}

impl Display for ServerInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "pid: {}, port: {}, scheme: {}, name: {}, command: {}, args: {}", self.pid, self.port, self.scheme, self.name, self.command, self.args.join(" "))
    }
}

impl ServerInfo {
    pub(crate) fn new(pid: i32, port: u16, scheme: String, name: String, command: String, args: Vec<String>) -> ServerInfo {
        ServerInfo {
            pid,
            port,
            scheme: scheme.clone(),
            name: name.clone(),
            command: command.clone(),
            args: args.clone(),
        }
    }
    pub(crate) fn pid(&self) -> i32 {
        self.pid
    }
    pub(crate) fn port(&self) -> u16 {
        self.port
    }

    pub(crate) fn scheme(&self) ->String{
        self.scheme.clone()
    }

}
