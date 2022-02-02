use std::env;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LocalState {
    pub archives: Vec<crate::archive::ArchiveInstallation>,
    pub packages: crate::pkg::Packages,
    pub repos: Vec<crate::repo::Repo>,
    pub settings: Settings,
}

#[derive(Deserialize)]
pub struct Settings {
    pub clone_dir: String,
    pub unpack_dir: String,
}

pub fn expand_user(path: &str) -> String {
    let home = env::var("HOME").unwrap();
    path.replace("~", &home)
}
