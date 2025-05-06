use std::collections::HashMap;

use anyhow::Error;
use lol_html::{HtmlRewriter, element};

pub fn rewrite_html(
    root_url: &str,
    asset_map: &HashMap<String, String>,
    input: &str,
) -> Result<String, Error> {
    let rewrite_url = |url: &str| -> Result<String, Error> {
        if url.starts_with("/public/") {
            let public_item = url.strip_prefix("/public/").unwrap();
            let hashed_item = asset_map
                .get(public_item)
                .ok_or(anyhow::anyhow!("could not find asset: {public_item}"))?;
            let new_ref = format!("{root_url}/public/{hashed_item}");
            return Ok(new_ref);
        } else if url.starts_with("/") {
            let new_ref = format!("{root_url}{url}");
            return Ok(new_ref);
        }
        return Ok(url.to_owned());
    };
    let mut output = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        lol_html::Settings {
            element_content_handlers: vec![
                element!("a[href],link[href]", |el| {
                    let href = el.get_attribute("href").expect("href was required");
                    el.set_attribute("href", &rewrite_url(&href)?)?;

                    Ok(())
                }),
                element!("img[src],script[src]", |el| {
                    let src = el.get_attribute("src").expect("src was required");
                    el.set_attribute("src", &rewrite_url(&src)?)?;
                    Ok(())
                }),
            ],
            ..lol_html::Settings::new()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );
    rewriter.write(input.as_bytes()).unwrap();
    Ok(String::from_utf8(output).unwrap())
}
