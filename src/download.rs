use ureq::Error;

pub fn get_reader(url: &str) -> Result<impl std::io::Read+Send, String> {
    let resp = ureq::get(url).call();
    match resp {
        Ok(resp) => return Ok(resp.into_reader()),
        Err(Error::Status(_code, response)) => {
            return Err(response.status_text().to_string())
        },
        Err(_) => {
            return Err("Transport error".to_string())
        }
    }
}
