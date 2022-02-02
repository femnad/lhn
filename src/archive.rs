use flate2::read::GzDecoder;
use serde::Deserialize;
use std::path::Path;
use std::process::Command;
use tar::Archive;

use crate::unless;
use crate::util;

#[derive(Deserialize)]
pub struct ArchiveInstallation {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    unless: unless::Unless,
    #[serde(default)]
    links: Vec<Link>,
    url: String,
}

#[derive(Deserialize)]
pub struct Link {
    src: String,
    #[serde(default)]
    dest: String,
}

impl ArchiveInstallation {
    fn replace_version(&self, text: &str) -> String {
        text.replace("${version}", &self.version)
    }

    pub fn get_unless(&self) -> &unless::Unless {
        &self.unless
    }

    pub fn get_url(&self) -> String {
        self.replace_version(&self.url)
    }
}

pub fn extract(reader: impl std::io::Read + Send, unpack_dir: &str, archive: ArchiveInstallation) {
    let tar = GzDecoder::new(reader);
    let mut tar = Archive::new(tar);
    tar.unpack(unpack_dir.clone()).unwrap();

    archive.links.iter().for_each(|link_spec| {
        let original = archive.replace_version(&link_spec.src);
        let original = Path::new(&unpack_dir).join(original);

        println!("{}", original.to_str().unwrap());
        let link = util::expand_user(&link_spec.dest.to_string());
        std::os::unix::fs::symlink(original, link).unwrap();
    });
}

pub fn extract_archives(archives: Vec<ArchiveInstallation>, settings: &crate::util::Settings) {
    for archive in archives {
        let url = &archive.get_url();
        let unless = archive.get_unless();
        let cmd_tokens = unless.cmd.as_str().split(' ').collect::<Vec<&str>>();
        let cmd = &cmd_tokens[0];
        let args = &cmd_tokens[1..];

        let cmd_output = Command::new(cmd).args(args).output();

        if let Ok(..) = cmd_output {
            let cmd_output = String::from_utf8(cmd_output.unwrap().stdout).unwrap();
            let post_proc = crate::post::run_op(cmd_output.as_str(), unless.post.as_str());
            if post_proc.eq(archive.version.as_str()) {
                println!("Skipping {}", url);
                continue;
            }
        }

        println!("Downloading {}", url);

        let unpack_dir = util::expand_user(&settings.unpack_dir);

        match crate::download::get_reader(url) {
            Ok(reader) => {
                extract(reader, &unpack_dir, archive);
            }
            Err(e) => {
                println!("Unable to download from {}, response {}", url, e)
            }
        }
    }
}
