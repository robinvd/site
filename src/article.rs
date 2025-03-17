use askama::Template;
use pulldown_cmark::{Event, HeadingLevel, Tag};

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a> {
    title: &'a str,
    body: &'a str,
}

fn get_markdown_title(markdown: &str) -> Option<String> {
    let mut iter = pulldown_cmark::Parser::new(markdown);
    while let Some(event) = iter.next() {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => match iter.next() {
                Some(Event::Text(s)) => return Some(s.into()),
                _ => return None,
            },
            _ => {}
        }
    }
    None
}

pub fn markdown_to_html(markdown: &str) -> String {
    // Create parser with example Markdown text.
    let parser = pulldown_cmark::Parser::new(markdown);

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    let title = get_markdown_title(markdown).expect("no h1 header in article");

    let html_output = ArticleTemplate {
        title: &title,
        body: &html_output,
    }
    .render()
    .unwrap();

    html_output
}
