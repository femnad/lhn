use std::path::Path;

use git2::Repository;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Repo {
    url: String,
}

fn get_name(url: &str) -> String {
    let name = url.split('/').last().unwrap();
    name.split('.').next().unwrap().to_string()
}

pub fn clone_repos(repos: Vec<Repo>, clone_dir: String) {
    for repo in repos {
        let name = get_name(&repo.url);
        let path = format!("{}/{}", clone_dir, name);

        let repo_path = Path::new(&path);

        if repo_path.exists() {
            println!("Path {} already exists", path);
            continue;
        }

        println!("Cloning {} to {}", repo.url, path);

        match Repository::clone(&repo.url, path) {
            Ok(_) => (),
            Err(e) => panic!("failed to clone {}: {}", repo.url, e),
        };
    }
}
