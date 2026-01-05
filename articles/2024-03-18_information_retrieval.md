---
title: Aggregation on Information Retrieval (IR)
# publish_date: 2024-03-18
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

### Base implementation for inverted index

You can think of an inverted index as a big map that maps from words to the document they appear in.

```python
documents = [
  "hello there",
  "i like rust",
  "there is nothing like it",
]

index: dict[str, set[int]] = {}
for document_index, document in enumerate(documents):
  for word in document.split():
    index.setdefault(word, set()).add(document_index)
```

Now `index["like"]` will be `{1, 2}`. As "like" appears in the second and third document.
To search using the using we iterate over the words in our query, and maintain a set of documents where they all appear in.

```python
query = "i rust"
results: set[int] | None = None
for word in query.split():
  documents_containing_word: set[int] = index[word]
  if results is None:
    results = documents_containing_word
  else:
    results = results.intersection(documents_containing_word)

print(results)
# {1}
```

This is all thats required for the most basic search index! On which most search engines build.
This can then be extended with storing the index on disk, adding rankings, etc.

### Implementation for signature base lookup

```python
# number of bits required for a bitset that fits all documents
document_len_bits = math.ceil(len(documents) / 8) * 8

DOCUMENT_BYTES = 4
bit_signatures = np.zeros((document_len_bits, DOCUMENT_BYTES), dtype=np.uint8)
for document_i, document in enumerate(documents):
    signature = bit_signatures[document_i]
    words_to_signature(document, signature)

sliced_bits = transpose_bits(bit_signatures)
query_signature = np.zeros(DOCUMENT_BYTES, dtype=np.uint8)
words_to_signature(query, query_signature)

result = np.full(document_len_bits // 8, (1 << 8) - 1, dtype=np.uint8)
for bit_index in bitarray_ones(query_signature):
    result &= sliced_bits[bit_index]

print(list(bitarray_ones(result)))
```

Notice how the setup is quite a lot more complicated, but the lookup is just this small section with only bitwise operations:

```python
query_signature = np.zeros(DOCUMENT_BYTES, dtype=np.uint8)
words_to_signature(query, query_signature)

result = np.full(document_len_bits // 8, (1 << 8) - 1, dtype=np.uint8)
for bit_index in bitarray_ones(query_signature):
    result &= sliced_bits[bit_index]

print(list(bitarray_ones(result)))
# [1]
```
