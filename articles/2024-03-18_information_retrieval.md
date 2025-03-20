---
title: Aggregation on Information Retrieval (IR)
publish_date: 2024-03-18
tags: [information-retrieval, aggregation]
---

The first (and maybe only) post in a new series where i note down various interesting media i've found/read/watches recently on a specific topic.
Largely just for myself to reference later when im looking for a related article.
This time on [Information Retrieval](https://en.wikipedia.org/wiki/Information_retrieval)!

I've had a course in uni about this, and was always interested in the topic. It combines interesting algorithms and structures with lowish level programming.
But in the theory we only discussed indexing using a [inverted index](https://en.wikipedia.org/wiki/Inverted_index). A simple but very powerfull approach.

## Signature based IR

I dont remember how i found the article [Building a custom code search index in Go for searchcode.com](https://boyter.org/posts/how-i-built-my-own-index-for-searchcode/).
But its a super interesting variant of full text search, using bloom filters instead of inverted indexes!
The advantage of bloom filters is way less memory used, but the answer is probabilistic.
And using bloom filters for IR works the same way. Thus you have to verify that the found documents are not false positives.

The searchcode post gives a great overview, and in a very approachable style.
But diving into the actual paper [bitfunnel](https://danluu.com/bitfunnel-sigir.pdf) cleared up any questions i had.
It also includes IR background and alternative approaches, multiple examples and extensions to the main algorithm. 

The searchcode article mentions using this for a possible js client side search engine. And i think that might be a fun
but useless addition to this site :).
