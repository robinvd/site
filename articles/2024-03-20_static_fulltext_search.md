---
tags: [site, information-retrieval, bloom-filters,]
publish_date: 2025-03-20
title: Fulltext search for a static website
---
I just started up experimenting with this website, and looking for a small but
interesting project to add. I recently read up on using bitsignatures for full
text search on the client side, so why not try that out! 

> !bumi_question Whats the point of full text search with only 2 articles?

HEY i got to start somewhere!

## Background

This site is compiled fully to static html, so that it can be deployed for free
on github pages. And i currently dont have any reason to switch away from fully
static website as it doesnt need any interactivity or monthly costs. That means
that all processing has to be done at build time. And that normally excludes
search. Unless we build a client side full text search.

Using trigrams and inverted indexes is the usual way to build full text search.
Making a quick script to extract and count trigrams from an article
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

I had to quickly find an article from a different blog (as this one doesnt
have any significant posts yet). A random article from without.boats has
1781 unique trigrams. These are basically stored in a big `dict[trigram,
list[ArticleId]]`. So each unique trigram in an article increases the
storage by atleast 4 bytes (u32) for the ArticleID. With 1000 articles
thats `1000*1781*4 = 7124000 = ~7142kb = ~7mb`.

> !bumi_question Thats not even that much!

True, for modern connection 7mb should not be take too long. But that still very
wasteful. And most importantly not fun. Also doing a quick calculation using
[Bloom filter calculator](https://hur.st/bloomfilter/?n=1781&p=0.01&m=&k=) with
1781 items and 1% false positive rate we get `17071 bits ~= 2133 bytes`, times
1000 for the amount of articles `2133 * 1000 = 2133000 ~= 2133 kb ~= 2mb`.
Definitly a significant reduction.

## Steps

I need a few components for this to work. And for the challange im implementing
everything from scratch

- Bloom filters
- Indexing articles
- Seaching
- Ranking

> !bumi_leaving So nothing is done yet?

Its still a work in progress :)

A requirement for client side code is of course that is runs on the
client. I'm most familiar with python and rust, and both unforunately dont
run in the browser. Rust can compile to wasm, so that leaves doing it in
javascript/typescript or rust and compile to wasm.

I'm leaning to rust and wasm, as the current blog code is also written in rust.

## To be continued
