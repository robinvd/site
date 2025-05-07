use base64ct::{Base64UrlUnpadded, Encoding};
use crossbeam_channel::unbounded;
use db::{Db, Diagnostic, Dir, File, Tag};
use eyre::{Context, Error, Report};
use html::rewrite_html;
use sha1::{Digest, Sha1};
use std::{
    cmp::Reverse,
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use article::{Metadata, render_article};

mod article;
mod db;
mod html;
mod templates;

#[salsa::tracked]
fn article_by_tag<'a>(db: &'a dyn Db, root: Dir) -> HashMap<Tag<'a>, Vec<File>> {
    let mut tags = HashMap::new();
    for article_path in all_articles(db, root).unwrap() {
        let file = db.input(article_path.to_owned()).unwrap();
        let (_html, metadata) = compile_article(db, file);
        for tag_text in metadata.tags {
            let tag = Tag::new(db, tag_text);
            tags.entry(tag.to_owned()).or_insert(Vec::new()).push(file)
        }
    }
    tags
}

#[salsa::tracked]
fn all_article_metadata<'a>(db: &'a dyn Db, root: Dir) -> Vec<(File, Metadata)> {
    let mut results = Vec::new();
    for article_path in all_articles(db, root).unwrap() {
        let file = db.input(article_path.to_owned()).unwrap();
        let (_html, metadata) = compile_article(db, file);
        results.push((file, metadata));
    }
    results
}

#[salsa::tracked]
fn tag_posts<'a>(db: &'a dyn Db, root: Dir, tag: Tag<'a>) -> Vec<File> {
    let all_tags = article_by_tag(db, root);
    let posts = all_tags.get(&tag).cloned().unwrap_or(Vec::new());
    posts
}

#[salsa::tracked]
fn all_tags<'a>(db: &'a dyn Db, root: Dir) -> Vec<Tag<'a>> {
    let all_tags = article_by_tag(db, root);
    let mut tags: Vec<_> = all_tags.keys().cloned().collect();
    tags.sort();
    tags
}

fn compile_tag<'a>(db: &'a dyn Db, root: Dir, tag: Tag) -> String {
    let posts = tag_posts(db, root, tag);

    let mut posts = posts
        .into_iter()
        .map(|post| {
            let (_, metadata) = compile_article(db, post);
            let url = article_url(db, post);
            (metadata, url)
        })
        .collect::<Vec<_>>();
    posts.sort_by_key(|item| Reverse(item.0.publish_date));
    let tag_html = templates::tags::render_tag_page(tag.name(db), &posts);
    let asset_map = compile_asset_map(db);
    let tag_html = rewrite_html("..", &asset_map, &tag_html).unwrap();
    tag_html
}

fn output_tags<'a>(db: &'a dyn Db, root: Dir, output_path: &Path) -> Result<(), Error> {
    let tags_output_path = output_path.join("tags");
    for tag in all_tags(db, root) {
        let tag_output_path = tags_output_path.join(format!("{}.html", tag.name(db)));
        let tag_html = compile_tag(db, root, tag);
        fs::write(tag_output_path, tag_html)?;
    }
    Ok(())
}

#[salsa::tracked]
fn compile_asset_map<'a>(_db: &'a dyn Db) -> HashMap<String, String> {
    // TODO convert to salsa
    public_dir().unwrap()
}

#[salsa::tracked]
fn compile_article<'a>(db: &'a dyn Db, article: File) -> (String, Metadata) {
    let data = article.text(db);
    let text = String::from_utf8_lossy(&data).to_string();

    let (html_text, metadata) = render_article(&text, article.path(db));
    let asset_map = compile_asset_map(db);
    let html_text = rewrite_html("..", &asset_map, &html_text).unwrap();
    (html_text, metadata)
}

fn article_url(db: &dyn Db, post: File) -> String {
    let mut path = post.path(db);
    path.set_extension("");
    let name = path.file_name().unwrap().to_str().unwrap().to_owned();
    format!("/articles/{name}.html")
}

fn tag_url(db: &dyn Db, tag: Tag) -> String {
    format!("/tags/{}.html", tag.name(db))
}

#[salsa::tracked]
fn output_file<'a>(db: &'a dyn Db, data: &'a [u8], output_path: &'a Path) {
    match std::fs::write(output_path, data) {
        Ok(_) => {}
        Err(err) => Diagnostic::push_error(
            db,
            output_path,
            Report::from(err).wrap_err(format!(
                "could not output to file {}",
                output_path.display()
            )),
        ),
    }
}

fn output_article(db: &dyn Db, input: &Path, output_path: &Path) {
    let file = match db.input(input.to_path_buf()) {
        Ok(file) => file,
        Err(err) => {
            Diagnostic::push_error(db, output_path, err);
            return;
        }
    };
    let (html_file, _metadata) = compile_article(db, file);
    let input_path: PathBuf = file.path(db);
    let mut html_path = output_path.join(input_path.file_name().unwrap());
    html_path.set_extension("html");

    output_file(db, html_file.as_ref(), &html_path);
}

fn output_articles(db: &dyn Db, root_dir: Dir, output_path: &Path) -> Result<(), Error> {
    let article_output_path = output_path.join("articles");
    for input in all_articles(db, root_dir)? {
        output_article(db, input, &article_output_path);
    }
    Ok(())
}

fn all_articles<'a>(db: &'a dyn Db, input: Dir) -> Result<&'a [PathBuf], Error> {
    let artitle_path = input.path(db).join("articles");
    Ok(db.dir(artitle_path)?.items(db))
}

#[salsa::tracked]
fn compile_home(db: &dyn Db, root_dir: Dir) -> String {
    let mut all_articles = all_article_metadata(db, root_dir);

    let asset_map = compile_asset_map(db);
    all_articles.sort_by(|left, right| left.1.publish_date.cmp(&right.1.publish_date).reverse());
    let arg: Vec<_> = all_articles
        .into_iter()
        .map(|(f, md)| (md, article_url(db, f)))
        .collect();
    let home_html = templates::home::render_home(&arg);
    let home_html = rewrite_html(".", &asset_map, &home_html)
        .context("coult not rewrite home")
        .unwrap();
    home_html
}

fn output_home(db: &dyn Db, root_dir: Dir, output_path: &Path) -> Result<(), Error> {
    let article_output_path = output_path.join("index.html");

    let home_html = compile_home(db, root_dir);
    fs::write(article_output_path, home_html).context("could not write index.html")?;

    Ok(())
}

/// Computes and outputes the full output dir
///
/// - articles (html + plain)
/// - static/public files
/// - tag pages
/// - search
/// - index page
#[salsa::tracked]
fn output_dir<'a>(db: &'a dyn Db, root_dir: Dir) {
    let output_path = Path::new("./output");

    if let Err(e) = output_articles(db, root_dir, output_path) {
        Diagnostic::push_error(db, Path::new(""), e);
    }
    if let Err(e) = output_tags(db, root_dir, output_path) {
        Diagnostic::push_error(db, Path::new(""), e);
    }
    if let Err(e) = output_home(db, root_dir, output_path) {
        Diagnostic::push_error(db, Path::new(""), e);
    }
}

fn main_watch() -> Result<(), Error> {
    let (tx, rx) = unbounded();
    let mut db = db::BlogDatabase::new_watch(tx);

    let root = db.dir(Path::new(".").to_path_buf())?;

    loop {
        output_dir(&db, root);

        let diagnostics = output_dir::accumulated::<Diagnostic>(&db, root);
        if diagnostics.is_empty() {
            println!("compiled");
        } else {
            for diagnostic in diagnostics {
                println!("{}", diagnostic.0);
            }
        }

        for log in db.logs.lock().unwrap().drain(..) {
            eprintln!("{log}");
        }

        // Wait for file change events, the output can't change unless the
        // inputs change.
        for event in rx.recv()?.unwrap() {
            eprintln!("file watch event: {event:?}");
            let path = match event.path.canonicalize() {
                Ok(p) => p,
                Err(err) => {
                    if err.kind() == io::ErrorKind::NotFound {
                        continue;
                    }
                    return Err(err).with_context(|| {
                        format!("Failed to canonicalize path {}", event.path.display())
                    });
                }
            };
            db.reload_path(&path)?;
        }
    }
}

fn main_salsa() -> Result<(), Error> {
    let mode = std::env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "run".to_owned());

    match mode.as_str() {
        "run" => {
            let db = db::BlogDatabase::new();

            let root = db.dir(Path::new(".").to_path_buf())?;

            output_dir(&db, root);

            let diagnostics = output_dir::accumulated::<Diagnostic>(&db, root);
            if diagnostics.is_empty() {
                println!("compiled");
            } else {
                for diagnostic in diagnostics {
                    println!("{}", diagnostic.0);
                }
            }

            for log in db.logs.lock().unwrap().drain(..) {
                eprintln!("{log}");
            }
            Ok(())
        }
        "watch" => main_watch(),
        _ => panic!("unknown mode. Options: [run|watch]"),
    }
}

fn public_input_files() -> Result<Vec<PathBuf>, Error> {
    let mut results = Vec::new();
    for entry in fs::read_dir("./public").context("could not read public directory")? {
        let entry = entry?;
        results.push(entry.path().to_owned());
    }
    // TODO search code
    // for entry in
    //     fs::read_dir("./search/dist/assets").context("could not read search dist directory")?
    // {
    //     let entry = entry?;
    //     results.push(entry.path().to_owned());
    // }

    Ok(results)
}

fn public_dir() -> Result<HashMap<String, String>, Error> {
    let mut map = HashMap::new();
    for entry in public_input_files().context("could not list public files")? {
        let file_name = entry.file_name().unwrap().to_str().unwrap().to_owned();
        let mut file = fs::File::open(&entry).context("could not read public file")?;
        let mut hasher = Sha1::new();
        io::copy(&mut file, &mut hasher).context("copy failed")?;
        let hash = hasher.finalize();

        let base64_hash: String = Base64UrlUnpadded::encode_string(&hash)
            .chars()
            .take(7)
            .collect();

        let hashed_name = format!("{}_{}", base64_hash, file_name);
        let mut output_path = PathBuf::from("./output/public");
        output_path.push(&hashed_name);

        fs::copy(&entry, &output_path).context("could not copy hashed file to output/public")?;
        map.insert(file_name.to_owned(), hashed_name);
    }
    Ok(map)
}

fn main() -> Result<(), Error> {
    fs::create_dir_all("./output/articles").context("could not create output dir")?;
    fs::create_dir_all("./output/tags").context("could not create output dir")?;
    fs::create_dir_all("./output/public").context("could not create output dir")?;
    main_salsa()?;
    return Ok(());

    // TODO
    // - search
    // - shared navbar code

    // let search_html = templates::search::render_seach();
    // let search_html =
    //     rewrite_html(".", &asset_map, &search_html).context("coult not rewrite search")?;
    // fs::write("output/search.html", search_html).context("could not write search.html")?;
}
