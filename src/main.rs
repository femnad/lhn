mod config;
mod download;
mod pkg;
mod post;
mod repo;

extern crate structopt;
extern crate ureq;

use std::env;
use std::process::{exit, Command};

use flate2::read::GzDecoder;
use serde::Deserialize;
use std::path::Path;
use structopt::StructOpt;
use tar::Archive;

#[derive(Deserialize)]
struct LocalState {
    archives: Vec<ArchiveInstallation>,
    packages: pkg::Packages,
    repos: Vec<repo::Repo>,
    settings: Settings,
}

#[derive(Deserialize)]
struct Settings {
    clone_dir: String,
    unpack_dir: String,
}

#[derive(Deserialize)]
struct Link {
    src: String,
    #[serde(default)]
    dest: String,
}

#[derive(Deserialize)]
struct ArchiveInstallation {
    #[serde(default)]
    version: String,
    #[serde(default)]
    unless: Unless,
    #[serde(default)]
    links: Vec<Link>,
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
        text.replace("${version}", &self.version)
    }

    fn get_unless(&self) -> &Unless {
        &self.unless
    }

    fn get_url(&self) -> String {
        self.replace_version(&self.url)
    }
}

fn expand_user(path: &String) -> String {
    let home = env::var("HOME").unwrap();
    path.replace("~", &home)
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

        let unpack_dir = expand_user(&settings.unpack_dir);

        match download::get_reader(url) {
            Ok(reader) => {
                let tar = GzDecoder::new(reader);
                let mut tar = Archive::new(tar);
                tar.unpack(unpack_dir.clone()).unwrap();

                archive.links.iter().for_each(|link_spec| {
                    let original = archive.replace_version(&link_spec.src);
                    let original = Path::new(&unpack_dir).join(original);

                    println!("{}", original.to_str().unwrap());
                    let link = expand_user(&link_spec.dest.to_string());
                    std::os::unix::fs::symlink(original, link).unwrap();
                });
            }
            Err(e) => {
                println!("Unable to download from {}, response {}", url, e)
            }
        }
    }

    pkg::install(local_state.packages);

    repo::clone_repos(local_state.repos, expand_user(&settings.clone_dir));
}
