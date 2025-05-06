use askama::Template;

use crate::article::Metadata;

#[derive(Template)]
#[template(path = "article.html")]
pub struct ArticleTemplate<'a> {
    pub body: &'a str,
    pub metadata: &'a Metadata,
}
