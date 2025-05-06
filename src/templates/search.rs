use askama::Template;

#[derive(Template)]
#[template(path = "search.html")]

struct SearchTemplate {}

pub fn render_seach() -> String {
    let output = SearchTemplate {}.render().unwrap();
    output
}
