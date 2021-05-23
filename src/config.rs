use std::fs;
use std::io;

use ureq::Error;

pub fn get_content(target: &str) -> Result<String, io::Error> {
    if target.starts_with("https://") {
        let resp = ureq::get(target).call();
        match resp {
            Ok(resp) => return Ok(resp.into_string().unwrap()),
            Err(Error::Status(_code, response)) => {
                return Err(io::Error::new(io::ErrorKind::Other, response.status_text()))
            },
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "Transport error")),
        }
    }

    return fs::read_to_string(target);
}
