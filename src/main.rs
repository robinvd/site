use std::{ffi::OsStr, fs, path::PathBuf};

const HTML_TEMPLATE: &'static str = include_str!("template.html");

fn markdown_to_html(filename: &str, markdown: &str) -> String {
    // Create parser with example Markdown text.
    let parser = pulldown_cmark::Parser::new(markdown);

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    dbg!(filename);
    let title = filename
        .splitn(2, "_")
        .skip(1)
        .next()
        .expect("filename has no _");
    let html_output = HTML_TEMPLATE
        .replace("{{TITLE}}", title)
        .replace("{{BODY}}", &html_output);

    html_output
}

type Err = Box<dyn std::error::Error>;

fn main() -> Result<(), Err> {
    fs::create_dir("./output")?;
    for file in fs::read_dir("./articles")? {
        let file = file?;
        println!("processing: {:?}", file.file_name());
        let path = file.path();
        if file.metadata()?.is_file() && path.extension() == Some(OsStr::new("md")) {}
        let md_text = fs::read_to_string(file.path())?;
        let html_text = markdown_to_html(&path.file_stem().unwrap().to_string_lossy(), &md_text);
        let mut output_path = PathBuf::new();
        output_path.push("./output");
        output_path.push(path.file_name().unwrap());
        output_path.set_extension("html");
        fs::write(output_path, html_text)?;
    }
    Ok(())
}
