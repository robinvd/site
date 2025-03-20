use anyhow::{Context, Error};
use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf};

use article::render_article;

mod article;
mod home;
mod tags;
mod utils;

fn main() -> Result<(), Error> {
    fs::create_dir_all("./output/articles").context("could not create output dir")?;
    fs::create_dir_all("./output/tags").context("could not create output dir")?;
    utils::copy("./public/", "./output").context("could not copy public files to output")?;

    let mut all_articles = Vec::new();

    for file in fs::read_dir("./articles").context("could not read articles directory")? {
        let file = file.context("could not read a file in articles directory")?;
        println!("processing: {:?}", file.file_name());
        let path = file.path();
        if file
            .metadata()
            .context("could not get file metadata")?
            .is_file()
            && path.extension() == Some(OsStr::new("md"))
        {}
        let md_text = fs::read_to_string(file.path()).context("could not read markdown file")?;
        let (html_text, metadata) = render_article(&md_text);
        let mut output_path = PathBuf::new();
        output_path.push("./output/articles");
        output_path.push(path.file_name().unwrap());
        output_path.set_extension("html");
        fs::write(&output_path, html_text).context("could not write HTML file")?;

        all_articles.push((
            metadata,
            output_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ))
    }

    all_articles.sort_by(|left, right| left.1.cmp(&right.1).reverse());
    let home_html = home::render_home(&all_articles);
    fs::write("output/index.html", home_html).context("could not write index.html")?;

    let mut by_tag = HashMap::new();
    for (metadata, path) in all_articles.iter() {
        for tag in &metadata.tags {
            by_tag
                .entry(tag.as_str())
                .or_insert(Vec::new())
                .push((metadata, path));
        }
    }
    for items in by_tag.values_mut() {
        items.sort_by(|left, right| left.1.cmp(&right.1).reverse());
    }
    let mut by_tag_flat: Vec<_> = by_tag.into_iter().collect();
    by_tag_flat.sort_by(|left, right| left.0.cmp(right.0));

    for (tag_name, articles) in by_tag_flat {
        let tag_html = tags::render_tag_page(tag_name, &articles);
        fs::write(format!("output/tags/{tag_name}.html"), tag_html)
            .context("could not write tag html")?;
    }

    Ok(())
}
