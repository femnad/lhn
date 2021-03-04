mod post;
mod pkg;

extern crate ureq;
extern crate structopt;

use std::env;
use std::fs::File;
use std::process::{Command, exit};

use flate2::read::GzDecoder;
use serde::Deserialize;
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
    version: String,
    unless: Unless,
    url: String,
}

#[derive(Deserialize)]
struct Unless {
    cmd: String,
    post: String,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "lhn")]
struct Opt {
    #[structopt(short = "f", long = "file", default_value = "lhn.yml")]
    file: String,
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
    let file = File::open(&opt.file).unwrap_or_else(|e| {
        eprintln!("Error opening file `{}`: {}", opt.file, e);
        exit(1);
    });

    let local_state: LocalState = serde_yaml::from_reader(file).unwrap();

    let settings = local_state.settings;
    let archives = local_state.archives;

    for archive in archives {
        let url = &archive.get_url();
        let resp = ureq::get(url).call();
        let unless = archive.get_unless();
        let cmd_tokens = unless.cmd.as_str().split(" ").collect::<Vec<&str>>();
        let cmd = &cmd_tokens[0];
        let args = &cmd_tokens[1..];

        let cmd_output = Command::new(cmd)
            .args(args)
            .output();

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
        let tar = GzDecoder::new(resp.into_reader());
        let mut tar = Archive::new(tar);
        tar.unpack(unpack_dir).unwrap();
    }

    pkg::install(local_state.packages);
}
