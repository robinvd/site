use crate::db::Db;
use eyre::{Context, Error};
use serde::Deserialize;
use std::{cmp::Reverse, path::Path};

pub const LINKS_PATH: &str = "links.yaml";

#[derive(Deserialize, Clone)]
pub struct LinkEntry {
    pub title: String,
    pub url: String,
    pub notes: String,
    pub added: jiff::civil::Date,
}

pub fn load_links(db: &dyn Db) -> Result<Vec<LinkEntry>, Error> {
    let links_path = Path::new(LINKS_PATH);
    let file = db
        .input(links_path.to_path_buf())
        .context("could not read links.yaml")?;
    let text = String::from_utf8_lossy(file.text(db)).to_string();
    let mut links: Vec<LinkEntry> =
        serde_yaml::from_str(&text).context("could not parse links.yaml")?;
    links.sort_by_key(|link| Reverse(link.added));
    Ok(links)
}
