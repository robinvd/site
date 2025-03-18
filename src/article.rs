use askama::Template;
use pulldown_cmark::{Event, HeadingLevel, Tag, TagEnd};
use std::io::Write as _;

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

const HIGHLIGHT_NAMES: &'static [&'static str] = &[
    "attribute",
    "boolean",
    "carriage-return",
    "comment",
    "comment.documentation",
    "constant",
    "constant.builtin",
    "constructor",
    "constructor.builtin",
    "embedded",
    "error",
    "escape",
    "function",
    "function.builtin",
    "keyword",
    "markup",
    "markup.bold",
    "markup.heading",
    "markup.italic",
    "markup.link",
    "markup.link.url",
    "markup.list",
    "markup.list.checked",
    "markup.list.numbered",
    "markup.list.unchecked",
    "markup.list.unnumbered",
    "markup.quote",
    "markup.raw",
    "markup.raw.block",
    "markup.raw.inline",
    "markup.strikethrough",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "string.regexp",
    "string.special",
    "string.special.symbol",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
];

const CLASS_NAMES: &'static [&'static str] = &[
    "attribute",
    "boolean",
    "carriage-return",
    "comment",
    "comment-documentation",
    "constant",
    "constant-builtin",
    "constructor",
    "constructor-builtin",
    "embedded",
    "error",
    "escape",
    "function",
    "function-builtin",
    "keyword",
    "markup",
    "markup-bold",
    "markup-heading",
    "markup-italic",
    "markup-link",
    "markup-link-url",
    "markup-list",
    "markup-list-checked",
    "markup-list-numbered",
    "markup-list-unchecked",
    "markup-list-unnumbered",
    "markup-quote",
    "markup-raw",
    "markup-raw-block",
    "markup-raw-inline",
    "markup-strikethrough",
    "module",
    "number",
    "operator",
    "property",
    "property-builtin",
    "punctuation",
    "punctuation-bracket",
    "punctuation-delimiter",
    "punctuation-special",
    "string",
    "string-escape",
    "string-regexp",
    "string-special",
    "string-special-symbol",
    "tag",
    "type",
    "type-builtin",
    "variable",
    "variable-builtin",
    "variable-member",
    "variable-parameter",
];

fn render_code(lang: &str, code: &str) -> String {
    let (language, highlights_query) = match lang {
        "rust" => (
            tree_sitter_rust::LANGUAGE.into(),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
        ),
        "python" => (
            tree_sitter_python::LANGUAGE.into(),
            tree_sitter_python::HIGHLIGHTS_QUERY,
        ),
        _ => panic!("unknown code fence {}", lang),
    };
    let mut highlighter_config = tree_sitter_highlight::HighlightConfiguration::new(
        language,
        lang,
        highlights_query,
        "",
        "",
    )
    .unwrap();
    highlighter_config.configure(HIGHLIGHT_NAMES);
    let mut highlighter = tree_sitter_highlight::Highlighter::new();
    let highlights = highlighter
        .highlight(&highlighter_config, code.as_bytes(), None, |_| None)
        .unwrap();

    let mut html_highlighter = tree_sitter_highlight::HtmlRenderer::new();
    html_highlighter
        .render(highlights, code.as_bytes(), &|h, text| {
            // println!(
            //     "h {h:?} {} {:?}",
            //     HIGHLIGHT_NAMES[h.0],
            //     std::str::from_utf8(text)
            // );
            write!(text, r#"class="{}" "#, CLASS_NAMES[h.0]).unwrap();
        })
        .unwrap();

    let mut output = String::new();
    output.push_str("<pre><code>");
    for line in html_highlighter.lines() {
        output.push_str(line);
    }
    output.push_str("</code></pre>");
    output
}

fn color_codeblocks<'a>(
    mut events: impl Iterator<Item = Event<'a>> + 'a,
) -> impl Iterator<Item = Event<'a>> + 'a {
    std::iter::from_fn(move || {
        let event = events.next()?;
        if let Event::Start(Tag::CodeBlock(code_block_kind)) = event {
            let mut code = String::new();
            while let Some(next_event) = events.next() {
                match next_event {
                    Event::Text(text) => {
                        code.push_str(&text);
                    }
                    Event::End(TagEnd::CodeBlock) => {
                        let lang = match code_block_kind {
                            pulldown_cmark::CodeBlockKind::Indented => "".into(),
                            pulldown_cmark::CodeBlockKind::Fenced(cow_str) => cow_str,
                        };
                        let html = render_code(&lang, &code);
                        return Some(Event::Html(html.into()));
                    }
                    _ => {
                        panic!("unknown type in code block")
                    }
                }
            }

            panic!("missing end code block tag")
        } else {
            Some(event)
        }
    })
}

pub fn render_article(markdown: &str) -> (String, String) {
    // Create parser with example Markdown text.
    let parser = pulldown_cmark::Parser::new(markdown);
    let token_stream = color_codeblocks(parser);
    // let token_stream = parser;

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, token_stream);

    let title = get_markdown_title(markdown).expect("no h1 header in article");

    let html_output = ArticleTemplate {
        title: &title,
        body: &html_output,
    }
    .render()
    .unwrap();

    (title, html_output)
}
