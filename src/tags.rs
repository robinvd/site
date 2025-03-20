use askama::Template;

#[derive(Template)]
#[template(path = "tag.html")]
struct TagTemplate<'a> {
    articles: &'a [(&'a crate::article::Metadata, &'a String)],
    tag_name: &'a str,
}

pub fn render_tag_page(
    tag_name: &str,
    articles: &[(&crate::article::Metadata, &String)],
) -> String {
    let output = TagTemplate { tag_name, articles }.render().unwrap();
    output
}
