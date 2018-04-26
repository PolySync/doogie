<!--
© 2018, Devin Smith <dsmith@polysync.io>

This file is part of Doogie

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
-->

# Doogie

## Overview
Doogie is a wrapper library around [cmark](https://github.com/commonmark/cmark), the `C` implementation of 
[CommonMark](http://commonmark.org/). It provides some implicit memory safety around node allocation not 
present in the `C` library.
 
## Getting Started

### Dependencies
* [libcmark](https://github.com/commonmark/cmark#installing)

### Building

`cargo build`

## Usage

### Examples

``` rust
use doogie::parse_document;

let document = "# My Great Document \
\
* Item 1 \
* Item 2 \
* Item 3";

let root = parse_document(document);

let renderer = root.capabilities.render.as_ref().unwrap();
println!("{}", renderer.render_xml());
```

### API

The API is organized around capability objects attached to Nodes. Specifically

* `NodeGetter` 
* `NodeSetter` 
* `NodeTraverser` 
* `StructuralMutator` 
* `NodeRenderer` 
* `NodeDestructor` 

Their documentation is best viewed in rustdoc. You can run `cargo doc --open` to view it in your browser.

## Tests

The tests are located inline with the module code. There is a mix of property based and traditional unit style tests 
that mainly provide coverage around Doogie's capability system and memory safety.

### Running Tests

`cargo test`

# License

© 2018, Devin Smith <dsmith@polysync.io>

[MIT](https://github.com/PolySync/doogie/blob/master/LICENSE)
