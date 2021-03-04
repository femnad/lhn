use std::fs::File;
use std::io::Read;
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Packages {
    common: Vec<String>,
    dnf: Vec<String>,
}

const OS_RELEASE_FILE: &str = "/etc/os-release";

fn get_os_id() -> String {
    let mut os_release = File::open(OS_RELEASE_FILE)
        .expect(format!("Unable to open OS release file {}", OS_RELEASE_FILE).as_str());
    let mut os_release_content = String::new();
    os_release.read_to_string(&mut os_release_content).unwrap();
    os_release_content
        .split("\n")
        .filter(|line| line.starts_with("ID="))
        .map(|line| line.split("=").last().unwrap())
        .nth(0)
        .expect("Unable to find OS ID")
        .to_string()
}

trait PackageManager {
    fn get_specialized_packages(&self, packages: Packages) -> Vec<String>;
    fn get_install_command(&self) -> Vec<&str>;

    fn get_package_list(&self, packages: Packages) -> Vec<String> {
        let mut package_list = packages.common.clone();
        package_list.extend(self.get_specialized_packages(packages));
        package_list
    }

    fn install(&self, packages: Packages) {
        let output = Command::new("sudo")
            .args(self.get_install_command())
            .args(self.get_package_list(packages))
            .output()
            .expect("error installing packages with dnf");
        println!("{}", String::from_utf8(output.stdout).unwrap());
    }
}

struct Dnf {}

impl PackageManager for Dnf {
    fn get_specialized_packages(&self, packages: Packages) -> Vec<String> {
        packages.dnf
    }

    fn get_install_command(&self) -> Vec<&str> {
        vec!["dnf", "install", "-y"]
    }
}

fn get_package_manager() -> Result<Box<dyn PackageManager>, String> {
    let os_id = get_os_id();
    match os_id.as_str() {
        "fedora" => Ok(Box::new(Dnf {})),
        _ => Err("Cannot determine package manager".to_string()),
    }
}

pub fn install(packages: Packages) {
    let package_manager = get_package_manager().unwrap();
    package_manager.install(packages);
}
