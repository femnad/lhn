extern crate structopt;
extern crate ureq;

use std::process::exit;

use structopt::StructOpt;

mod archive;
mod config;
mod download;
mod pkg;
mod post;
mod repo;
mod unless;
mod util;

#[derive(Debug, StructOpt)]
#[structopt(name = "lhn")]
struct Opt {
    #[structopt(short = "f", long = "config", default_value = "lhn.yml")]
    config: String,
}

fn main() {
    let opt = Opt::from_args();

    let content = config::get_content(&opt.config).unwrap_or_else(|e| {
        eprintln!("Error reading config `{}`: {}", opt.config, e);
        exit(1);
    });

    let local_state: util::LocalState = serde_yaml::from_str(&content).unwrap();

    archive::extract_archives(local_state.archives, &local_state.settings);

    pkg::install(local_state.packages);

    repo::clone_repos(local_state.repos, util::expand_user(&local_state.settings.clone_dir));
}
