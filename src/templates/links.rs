use askama::Template;

#[derive(Template)]
#[template(path = "links.html")]
struct LinksTemplate<'a> {
    links: &'a [crate::links::LinkEntry],
}

pub fn render_links(links: &[crate::links::LinkEntry]) -> String {
    LinksTemplate { links }.render().unwrap()
}
