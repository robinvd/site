---
tags: [rust, programming]
publish_date: 2026-01-05
title: Reflections on edit and owning the full stack
---

I recently came across Microsoft's [Edit](https://github.com/microsoft/edit) program.
It's quite an odd program and codebase: both small and featureful.
I also find the subtitle hilarious: `We all edit.`
But I mostly want to muse on the code style and dependencies, or lack thereof.


Its code style is weird in a good way.
It defines almost no generic structs or functions, while still having good abstractions.
The way the code is written is extremely procedural and reminds me a lot of C, which I think the author is very experienced with.
But this C style, plus small Rust additions, makes it super easy to read and understand.


I learned Rust through Haskell, and both languages and their popular libraries love to abstract things and make everything generic and reusable. But there is a beauty in non-generic, simple, procedural code.

A second thing that comes from these same principles is that it uses almost no dependencies. Compiling the program takes almost no time. There is nothing to download, no complex nested trait calls to solve, no code to parse and check that then doesn't get used.

As a text editor, it does have implementations of some potentially complex things like
- a text buffer suitable for an editor
- an immediate-mode UI with layout and table support
- a framebuffer implementation
- full VT100 terminal escape code parsing
- cross-platform support for Unix and Windows
- completely custom Unicode and UTF-8 parsing and measuring

But for most of these the implementation complexity is not that bad, as they are incredibly limited and tuned to the needs of the program:
- it uses the simplest buffer for a text editor: a gap buffer. This could cause performance issues when using multiple cursors or concurrent access. But Edit does not have these features
- its UI layout, while impressive, does not implement nearly enough to come close to CSS or similar. Again, only layout that the editor needs

This reframed how I think about owning the whole stack, and the simplicity when no extra features are required.
This leaves it with such a simple codebase, where each module can be quickly understood and improved without having to onboard for weeks.

There are some very clear disadvantages to making and owning the whole stack: You have to make and maintain everything.
Even in Edit these are already clear. When I was looking into it, the layout code was clearly in its initial stages.
Proper percentages or fractional units (`fr` in CSS Grid) weren't implemented yet, but the current UI also didn't need them.

As I was so captured by this simple yet powerful program, I made a hobby conversion from Rust to Go. For fun, and to learn some more terminal / text editor / Go skills. I kept at this until it had a usable version, but at the `integration hell` stage I gave up on it, as I don't actually have any use for it.
But the idea of low-dependency projects and owning the full stack stuck with me, and this gave rise to my current programming obsession: TUIs.
None of the currently popular frameworks give me what I needed. So I set out to make my own, with heavy influence from Edit.

So far I have made a terminal git diff viewer and a terminal multiplexer (a subset of tmux functionality).
It all needs a good cleanup / README etc before it can be announced and put online.
Just like Edit, there is full layout support, and full mouse support! And of course minimal dependencies.
So far the only dependencies in the base TUI library are [parsing VTE escape codes](https://github.com/helix-editor/termina), a [flexbox layout engine](https://github.com/DioxusLabs/taffy), and an [async runtime](https://github.com/smol-rs/smol).
These three are complex enough to take significant time investment while not having a large design space to innovate in.
