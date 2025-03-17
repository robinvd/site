use askama::Template;

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a> {
    title: &'a str,
    body: &'a str,
}

pub fn markdown_to_html(filename: &str, markdown: &str) -> String {
    // Create parser with example Markdown text.
    let parser = pulldown_cmark::Parser::new(markdown);

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    let title = filename
        .splitn(2, "_")
        .skip(1)
        .next()
        .expect("filename has no _");

    let html_output = ArticleTemplate {
        title,
        body: &html_output,
    }
    .render()
    .unwrap();

    html_output
}
