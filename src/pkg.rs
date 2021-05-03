extern crate regex;

use std::fs::File;
use std::io::Read;
use std::process::Command;

use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::ops::Deref;

const OS_RELEASE_FILE: &str = "/etc/os-release";
const NON_WHITESPACE_PATTERN :&str = r"[^\s]+";

#[derive(Deserialize)]
pub struct Packages {
    common: Vec<String>,
    apt: Vec<String>,
    dnf: Vec<String>,
}

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
    fn get_installed(&self, packages: Vec<String>) -> Vec<String>;

    fn get_non_installed(&self, packages: Packages) -> Vec<String> {
        let packages = self.get_package_list(packages);

        let installed: HashSet<String> = self.get_installed(packages.clone()).into_iter().collect();

        let non_installed: HashSet<String> = packages
            .into_iter()
            .collect();

        let missing = non_installed.difference(&installed);
        return missing
            .map(|p| p.clone())
            .collect::<Vec<_>>()
    }

    fn get_package_list(&self, packages: Packages) -> Vec<String> {
        let mut package_list = packages.common.clone();
        package_list.extend(self.get_specialized_packages(packages));
        package_list
    }

    fn install(&self, packages: Packages) {
        let packages_to_install = self.get_non_installed(packages);
        if packages_to_install.len() == 0 {
            println!("No packages to install");
            return;
        }

        println!("Installing packages: {}", packages_to_install.join(", "));
        Command::new("sudo")
            .args(self.get_install_command())
            .args(packages_to_install)
            .output()
            .expect("error installing packages");
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

    fn get_installed(&self, packages: Vec<String>) -> Vec<String> {
        let output = Command::new("dnf")
            .args(vec!["list", "installed"])
            .args(packages)
            .output()
            .expect("error listing installed packages with dnf");

        let output = String::from_utf8(output.stdout).unwrap();
        return output.split("\n")
            .skip(1) // header
            .map(|line| {
                //<package>.<arch>
                String::from(line.split(".").nth(0).unwrap())
            })
            .collect();
    }
}

struct Apt {}

impl PackageManager for Apt {
    fn get_specialized_packages(&self, packages: Packages) -> Vec<String> {
        packages.apt
    }

    fn get_install_command(&self) -> Vec<&str> {
        vec!["apt", "install", "-y"]
    }

    fn get_installed(&self, packages: Vec<String>) -> Vec<String> {
        let output = Command::new("dpkg-query")
            .arg("--list")
            .arg("--no-pager")
            .args(packages)
            .output()
            .expect("error listing installed packages with apt");

        let output = String::from_utf8(output.stdout).unwrap();

        let field_pattern = Regex::new(NON_WHITESPACE_PATTERN).unwrap();

        let fail = output.trim().split("\n")
            .skip(5) // headers and separator
            .map(|line| {
                let fields = field_pattern.captures_iter(line)
                    .map(|c| c.get(0).unwrap().as_str())
                    .collect::<Vec<&str>>();
                //<status><err> <name> <version> <arch> <desc>
                let name = fields.get(1).unwrap().deref();
                String::from(name)
            })
            .collect();
        return fail;
    }
}

fn get_package_manager() -> Result<Box<dyn PackageManager>, String> {
    let os_id = get_os_id();
    match os_id.as_str() {
        "fedora" => Ok(Box::new(Dnf {})),
        "debian" | "ubuntu" => Ok(Box::new(Apt {})),
        _ => Err("Cannot determine package manager".to_string()),
    }
}

pub fn install(packages: Packages) {
    let package_manager = get_package_manager().unwrap();
    package_manager.install(packages);
}
