use askama::Template;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};
use serde::Deserialize;
use std::io::Write as _;

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a> {
    body: &'a str,
    metadata: &'a Metadata,
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

fn handle_code_block<'a>(
    code_block_kind: &CodeBlockKind<'a>,
    mut events: &'a [Event<'a>],
    mut emit: impl FnMut(Event<'a>),
) -> &'a [Event<'a>] {
    let mut code = String::new();
    while events.len() > 0 {
        let next_event = &events[0];
        events = &events[1..];
        match next_event {
            Event::Text(text) => {
                code.push_str(&text);
            }
            Event::End(TagEnd::CodeBlock) => {
                let lang = match code_block_kind {
                    pulldown_cmark::CodeBlockKind::Indented => "".into(),
                    pulldown_cmark::CodeBlockKind::Fenced(cow_str) => cow_str.clone(),
                };
                let html = render_code(&lang, &code);
                emit(Event::Html(html.into()));
                return events;
            }
            _ => {
                panic!("unknown type in code block")
            }
        }
    }
    panic!("no end codeblock")
}

#[derive(Deserialize, Default)]
pub struct Metadata {
    pub tags: Vec<String>,
    #[serde(default)]
    pub title: String,
    pub publish_date: Option<jiff::civil::Date>,
}

fn postprocess_events<'a>(mut events: &'a [Event<'a>]) -> (Vec<Event<'a>>, Metadata) {
    let mut new = Vec::new();
    let mut stack: Vec<(Tag<'a>, Option<Box<dyn FnOnce() -> Vec<Event<'a>>>>)> = Vec::new();
    let mut metadata = Metadata::default();
    while events.len() > 0 {
        println!("e: {:?}", events[0]);
        match events {
            [Event::Start(Tag::MetadataBlock(_)), ..] => {
                let mut metadata_text = String::new();
                for (event_i, event) in events[1..].iter().enumerate() {
                    match event {
                        Event::End(TagEnd::MetadataBlock(_)) => {
                            metadata = serde_yaml::from_str(&metadata_text).unwrap();
                            events = &events[event_i + 2..];
                            break;
                        }
                        Event::Text(cow_str) => {
                            metadata_text.push_str(cow_str);
                        }
                        e => panic!("unknown event in metadata {:?}", e),
                    }
                }
            }
            [Event::Start(Tag::CodeBlock(code_kind)), rest @ ..] => {
                events = handle_code_block(code_kind, rest, |event| new.push(event));
            }
            [
                s_quote @ Event::Start(quote_tag @ Tag::BlockQuote(_)),
                s_paragraph @ Event::Start(Tag::Paragraph),
                Event::Text(text),
                rest @ ..,
            ] if text.starts_with("!") => {
                stack.push((quote_tag.clone(), None));
                stack.push((
                    Tag::Paragraph,
                    Some(Box::new(|| {
                        vec![Event::InlineHtml(format!("</div>").into())]
                    })),
                ));

                let text = text.clone().into_string();
                let (quote_type, quote_rest) = text.split_once(" ").unwrap_or((&text, ""));
                let quote_type = quote_type.strip_prefix("!").unwrap();
                let img_src = match quote_type {
                    "bumi_question" => "../bumi_question.png",
                    "bumi_leaving" => "../bumi_leaving.png",
                    ty => panic!("unknown quote type: {}", ty),
                };
                new.push(s_quote.clone());
                new.push(Event::InlineHtml(
                    format!(r#"<div class="cat_quote"><img src="{img_src}">"#).into(),
                ));
                new.push(s_paragraph.clone());
                if quote_rest.len() > 0 {
                    new.push(Event::Text(CowStr::Boxed(quote_rest.into())));
                }
                events = rest
            }
            [event @ Event::Start(tag), rest @ ..] => {
                stack.push((tag.clone(), None));
                new.push(event.clone());
                events = rest
            }
            [event @ Event::End(tag), rest @ ..] => {
                let (start, callback) = stack.pop().unwrap();
                assert_eq!(start.to_end(), *tag);
                new.push(event.clone());
                if let Some(callback) = callback {
                    new.extend(dbg!(callback()));
                }
                events = rest
            }
            [event, rest @ ..] => {
                new.push(event.clone());
                events = rest
            }
            [] => return (new, metadata),
        }
    }
    (new, metadata)
}

pub fn render_article(markdown: &str) -> (String, Metadata) {
    // Create parser with example Markdown text.
    let parser: pulldown_cmark::Parser = pulldown_cmark::Parser::new_with_broken_link_callback(
        markdown,
        pulldown_cmark::Options::ENABLE_TABLES
            | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
            | pulldown_cmark::Options::ENABLE_TASKLISTS
            | pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION
            | pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
            | pulldown_cmark::Options::ENABLE_WIKILINKS,
        None,
    );
    let all_events: Vec<_> = parser.collect();
    let (token_stream, mut metadata) = postprocess_events(&all_events);
    // let token_stream = parser;

    // Write to a new String buffer.
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, token_stream.into_iter());

    let html_output = ArticleTemplate {
        body: &html_output,
        metadata: &metadata,
    }
    .render()
    .unwrap();

    (html_output, metadata)
}
