use std::{ffi::OsStr, fs, path::PathBuf};

use article::render_article;

mod article;
mod home;
mod utils;

type Err = Box<dyn std::error::Error>;

fn main() -> Result<(), Err> {
    fs::create_dir_all("./output/articles")?;
    utils::copy("./public/", "./output")?;

    let mut all_articles = Vec::new();

    for file in fs::read_dir("./articles")? {
        let file = file?;
        println!("processing: {:?}", file.file_name());
        let path = file.path();
        if file.metadata()?.is_file() && path.extension() == Some(OsStr::new("md")) {}
        let md_text = fs::read_to_string(file.path())?;
        let (article_title, html_text) = render_article(&md_text);
        let mut output_path = PathBuf::new();
        output_path.push("./output/articles");
        output_path.push(path.file_name().unwrap());
        output_path.set_extension("html");
        fs::write(&output_path, html_text)?;

        all_articles.push((
            article_title,
            output_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ))
    }

    all_articles.sort_by(|left, right| left.1.cmp(&right.1).reverse());
    let home_html = home::render_home(&all_articles);
    fs::write("output/index.html", home_html)?;

    Ok(())
}
