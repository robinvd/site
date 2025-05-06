---
tags: [site, information-retrieval, bloom-filters]
publish_date: 2025-03-20
title: Fulltext search for a static website
---
I just started experimenting with this website and was looking for a small but interesting project to add. I recently read about using bit signatures for full-text search on the client side, so why not try that out?

> !bumi_question Whats the point of full text search with only 2 posts on the website?

HEY i got to start somewhere!

## Background

This site is fully compiled to static HTML, so it can be deployed for free on GitHub Pages. I currently have no reason to switch away from a fully static website, as it doesn't require interactivity or monthly costs. This means all processing has to be done at build time, which normally excludes search. Unless we build a client-side full-text search. Using trigrams and inverted indexes is the usual way to build full text search. Making a quick script to extract and count trigrams from an article
```python
from collections.abc import Iterator
import sys
import re

text = sys.stdin.read()
words: list[str] = re.split(r"\W+", text)


def to_trigrams(word: str) -> Iterator[str]:
    for i in range(len(word) - 2):
        yield word[i : i + 3]


trigrams: set[str] = set()
for word in words:
    trigrams.update(to_trigrams(word))

print(len(trigrams))
```

I tested this script on an article from another blog (since this one doesn't have significant posts yet). A random article from without.boats has 1781 unique trigrams. These are stored in a big `dict[trigram, list[ArticleId]]`. Each unique trigram in an article increases storage by at least 4 bytes (u32) for the ArticleID. With 1000 articles, that's `1000 * 1781 * 4 = 7124000 = ~7142 KB = ~7 MB`.

> !bumi_question Thats not even that much!

True, for modern connections, 7 MB isn't too much. But it's still wasteful and, most importantly, not fun. Using a quick calculation with the [Bloom filter calculator](https://hur.st/bloomfilter/?n=1781&p=0.01&m=&k=), we find that with 1781 items and a 1% false positive rate, we need `17071 bits ~= 2133 bytes`. For 1000 articles, that's `2133 * 1000 = 2133000 ~= 2133 KB ~= 2 MB`. Definitely a significant reduction.

## Steps

To make this work, I need to implement a few components. For the challenge, I'm building everything from scratch:

- Bloom filters
- Indexing articles
- Searching
- Ranking

> !bumi_leaving So nothing is done yet?

Not yet—it's still a work in progress!

A requirement for client-side code is, of course, that it runs on the client. I'm most familiar with Python and Rust, but unfortunately, neither runs natively in the browser. Rust can compile to WebAssembly (Wasm), so that leaves me with two options: JavaScript/TypeScript or Rust compiled to Wasm.

I'm leaning toward Rust and Wasm, as the current blog code is also written in Rust.

## To be continued
