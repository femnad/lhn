use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Unless {
    #[serde(default)]
    pub cmd: String,
    #[serde(default)]
    pub post: String,
}
