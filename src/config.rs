use std::fs;
use std::io;

extern crate ureq;

pub fn get_content(target: &str) -> Result<String, io::Error>{
    if target.starts_with("https://") {
        let resp = ureq::get(target).call();
        if !resp.ok() {
            return Err(io::Error::new(io::ErrorKind::Other, resp.status_line()));
        }
        return resp.into_string();
    }

    return fs::read_to_string(target);
}
