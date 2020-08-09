use regex::Regex;
use std::Copy;

pub(crate) enum PhpServerSapi {
    FPM,
    CGI,
    CLI,
    Unknown,
}

impl PhpServerSapi {
    pub(crate) fn from_str(str: &str) -> PhpServerSapi {
        match str {
            "FPM" => PhpServerSapi::FPM,
            "CLI" => PhpServerSapi::CLI,
            "CGI" => PhpServerSapi::CGI,
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

//
//
//
//
//
//
//
//

#[derive(Hash, Eq, PartialEq)]
pub(crate) struct PhpVersion<'a> {
    _version: &'a str,
}

impl PhpVersion {
    pub(crate) fn from_string(version: String) -> PhpVersion {
        PhpVersion::from_str(version.as_str())
    }

    pub(crate) fn from_str(version: &str) -> PhpVersion {
        let version_regex = Regex::new("^[578]\\.\\d{1,2}\\.\\d+$").unwrap();

        if !version_regex.is_match(&version) {
            panic!("Version \"{}\" is not a valid PHP version.", &version);
        }

        PhpVersion {
            _version: version.clone(),
        }
    }

    pub(crate) fn version(&self) -> &str {
        self._version.as_str()
    }
}

impl Copy for PhpVersion {
    fn clone(&self) -> PhpVersion {
        *self
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

#[derive(Hash, Eq, PartialEq)]
pub(crate) struct PhpBinary {
    _version: PhpVersion,
    directory: String,
    cli: String,
    fpm: String,
    cgi: String,
    default: bool,
}

impl PhpBinary {
    pub(crate) fn from_version(version: PhpVersion) -> PhpBinary {
        PhpBinary {
            _version: version,
            directory: String::from(""),
            cli: String::from(""),
            fpm: String::from(""),
            cgi: String::from(""),
            default: false,
        }
    }

    pub(crate) fn set_directory(&mut self, directory: String) {
        self.directory = directory.clone();
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
        self._version.as_str()
    }

    pub(crate) fn path(&self) -> &str {
        self._path.to_str().unwrap()
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
            PhpServerSapi::Unknown => (),
        }
    }

    pub(crate) fn sapi_path(&self, sapi: &PhpServerSapi) -> String {
        match sapi {
            PhpServerSapi::FPM => self.fpm.clone(),
            PhpServerSapi::CLI => self.cli.clone(),
            PhpServerSapi::CGI => self.cgi.clone(),
            PhpServerSapi::Unknown => false,
        }
    }
}
