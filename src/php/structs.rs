use std::path::PathBuf;

pub(crate) enum PhpServerSapi {
    #[cfg(not(target_os = "windows"))]
    FPM,
    CGI,
    CLI,
    Unknown
}

pub(crate) struct PhpVersion {
    _version: String
}

impl PhpVersion {
    pub(crate) fn from_version(version: &String) -> PhpVersion {
        PhpVersion {
            _version: version.clone()
        }
    }

    pub(crate) fn version(&self) -> &str {
        self._version.as_str()
    }
}

pub(crate) struct PhpBinary {
    _version: String,
    _path: PathBuf,
    _sapi: PhpServerSapi
}

impl PhpBinary {
    pub(crate) fn from(version: String, path: PathBuf, sapi: PhpServerSapi) -> PhpBinary {
        PhpBinary {
            _version: version,
            _path: path,
            _sapi: sapi
        }
    }

    pub(crate) fn version(&self) -> &str {
        self._version.as_str()
    }

    pub(crate) fn path(&self) -> &str {
        self._path.as_str()
    }
}
