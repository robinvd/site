use std::path::PathBuf;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate<'a> {
    articles: &'a [(String, String)],
}

pub fn render_home(articles: &[(String, String)]) -> String {
    let output = HomeTemplate { articles }.render().unwrap();
    output
}
