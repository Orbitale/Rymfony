use std::{env, path::PathBuf};
use std::error::Error;
use std::fmt;
use std::fs::create_dir;
use std::result::Result;

use dirs::home_dir;
use sha2::Digest;

#[derive(Debug)]
struct ProjectFolderError(String);

impl fmt::Display for ProjectFolderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured: {}", self.0)
    }
}

impl Error for ProjectFolderError {}

pub(crate) fn get_rymfony_project_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = home_dir().unwrap().display().to_string();
    let homestr = home.as_str();



    if homestr != "" {
        let cwd = env::current_dir().unwrap();

        let mut hasher = sha2::Sha256::new();
        hasher.update(&cwd.to_str().unwrap().as_bytes());
        let hash = hasher.finalize();

        let rymfony_project_path = PathBuf::from(homestr)
            .join(".rymfony")
            .join(format!("{:x}", hash));

        if !rymfony_project_path.is_dir() {
            create_dir(&rymfony_project_path).expect(format!("Unable to make directory for project {}", rymfony_project_path.to_str().unwrap()).as_str());
        }

        return Ok(rymfony_project_path);
    }

    Err(Box::new(ProjectFolderError("Cannot find the \"HOME\" directory".into())))
}
