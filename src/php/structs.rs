use regex::Regex;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

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

    pub(crate) fn from_str(version: &str) -> PhpVersion {
        PhpVersion::from_string(version.to_string())
    }

    pub(crate) fn version(&self) -> &str {
        self._version.as_str()
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
pub(crate) struct PhpBinary {
    cli: String,
    fpm: String,
    cgi: String,
    default: bool,
    _version: PhpVersion,
}

impl PhpBinary {
    pub(crate) fn clone(&self) -> PhpBinary {
        PhpBinary {
            _version: self._version.clone(),
            cli: self.cli.clone(),
            fpm: self.fpm.clone(),
            cgi: self.cgi.clone(),
            default: self.default.clone(),
        }
    }

    pub(crate) fn from_version(version: PhpVersion) -> PhpBinary {
        PhpBinary {
            _version: version,
            cli: String::from(""),
            fpm: String::from(""),
            cgi: String::from(""),
            default: false,
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

    pub(crate) fn set_default(&mut self, default: bool) {
        self.default = default;
    }

    pub(crate) fn version(&self) -> &str {
        self._version.version()
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
            },
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
