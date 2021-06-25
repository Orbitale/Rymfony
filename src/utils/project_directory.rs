use std::env;
use std::error::Error;
use std::fmt;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::result::Result;

use dirs::home_dir;
use sha2::Digest;

#[derive(Debug)]
struct ProjectDirectoryError(String);

impl fmt::Display for ProjectDirectoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured: {}", self.0)
    }
}

impl Error for ProjectDirectoryError {}

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
            create_dir_all(&rymfony_project_path).expect(
                format!(
                    "Unable to create directory for project {}",
                    rymfony_project_path.to_str().unwrap()
                )
                .as_str(),
            );
        }

        create_log_directory(&rymfony_project_path);

        return Ok(rymfony_project_path);
    }

    Err(Box::new(ProjectDirectoryError(
        "Cannot find the \"HOME\" directory".into(),
    )))
}

fn create_log_directory(rymfony_project_path: &PathBuf) {
    let log_dir = rymfony_project_path.join("log");

    if !log_dir.is_dir() {
        create_dir_all(&log_dir).expect(
            format!(
                "Unable to create logs directory for project {}",
                log_dir.to_str().unwrap()
            )
            .as_str(),
        );
    }

}