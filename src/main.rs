mod config;
mod download;
mod pkg;
mod post;

extern crate structopt;
extern crate ureq;

use std::env;
use std::process::{exit, Command};

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use structopt::StructOpt;
use tar::Archive;

#[derive(Deserialize)]
struct LocalState {
    archives: Vec<ArchiveInstallation>,
    packages: pkg::Packages,
    settings: Settings,
}

#[derive(Deserialize)]
struct Settings {
    unpack_dir: String,
}

#[derive(Deserialize)]
struct ArchiveInstallation {
    #[serde(default)]
    version: String,
    #[serde(default)]
    unless: Unless,
    #[serde(default)]
    link: Vec<HashMap<String, String>>,
    url: String,
}

#[derive(Deserialize, Default)]
struct Unless {
    #[serde(default)]
    cmd: String,
    #[serde(default)]
    post: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "lhn")]
struct Opt {
    #[structopt(short = "f", long = "config", default_value = "lhn.yml")]
    config: String,
}

impl ArchiveInstallation {
    fn replace_version(&self, text: &String) -> String {
        return text.replace("${version}", &self.version);
    }

    fn get_unless(&self) -> &Unless {
        return &self.unless;
    }

    fn get_url(&self) -> String {
        return self.replace_version(&self.url);
    }
}

fn main() {
    let opt = Opt::from_args();

    let content = config::get_content(&opt.config).unwrap_or_else(|e| {
        eprintln!("Error reading config `{}`: {}", opt.config, e);
        exit(1);
    });

    let local_state: LocalState = serde_yaml::from_str(&content).unwrap();

    let settings = local_state.settings;
    let archives = local_state.archives;

    for archive in archives {
        let url = &archive.get_url();
        let unless = archive.get_unless();
        let cmd_tokens = unless.cmd.as_str().split(" ").collect::<Vec<&str>>();
        let cmd = &cmd_tokens[0];
        let args = &cmd_tokens[1..];

        let cmd_output = Command::new(cmd).args(args).output();

        if cmd_output.is_ok() {
            let cmd_output = String::from_utf8(cmd_output.unwrap().stdout).unwrap();
            let post_proc = post::run_op(cmd_output.as_str(), unless.post.as_str());
            if post_proc.eq(archive.version.as_str()) {
                println!("Skipping {}", url);
                continue;
            }
        }

        println!("Downloading {}", url);

        let home = env::var("HOME").unwrap();
        let unpack_dir = settings.unpack_dir.replace("~", &home);

        match download::get_reader(url) {
            Ok(reader) => {
                let tar = GzDecoder::new(reader);
                let mut tar = Archive::new(tar);
                tar.unpack(unpack_dir.clone()).unwrap();

                archive.link.iter().for_each(|link_spec| {
                    link_spec.iter().for_each(|(original, link)| {
                        let original = archive.replace_version(original);
                        let original = Path::new(&unpack_dir).join(original);

                        println!("{}", original.to_str().unwrap());
                        let link = link.replace("~", home.as_str());
                        std::os::unix::fs::symlink(original, link).unwrap();
                    })
                });
            },
            Err(e) => {
            println!(
                "Unable to download from {}, response {}",
                url,
                e)
            },
        }

    }

    pkg::install(local_state.packages);
}
