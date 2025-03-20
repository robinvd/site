use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate<'a> {
    articles: &'a [(crate::article::Metadata, String)],
}

pub fn render_home(articles: &[(crate::article::Metadata, String)]) -> String {
    let output = HomeTemplate { articles }.render().unwrap();
    output
}
