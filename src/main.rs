use std::{ffi::OsStr, fs, path::PathBuf};

use article::markdown_to_html;

mod article;
mod utils;

type Err = Box<dyn std::error::Error>;

fn main() -> Result<(), Err> {
    fs::create_dir_all("./output/articles")?;
    utils::copy("./public/", "./output")?;
    for file in fs::read_dir("./articles")? {
        let file = file?;
        println!("processing: {:?}", file.file_name());
        let path = file.path();
        if file.metadata()?.is_file() && path.extension() == Some(OsStr::new("md")) {}
        let md_text = fs::read_to_string(file.path())?;
        let html_text = markdown_to_html(&path.file_stem().unwrap().to_string_lossy(), &md_text);
        let mut output_path = PathBuf::new();
        output_path.push("./output/articles");
        output_path.push(path.file_name().unwrap());
        output_path.set_extension("html");
        fs::write(output_path, html_text)?;
    }
    Ok(())
}
