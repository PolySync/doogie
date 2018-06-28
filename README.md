<!--
© 2018, PolySync Technologies, Inc., Devin Smith <dsmith@polysync.io>

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
Doogie is a wrapper library around [cmark](https://github.com/commonmark/cmark),
the `C` implementation of [CommonMark](http://commonmark.org/). It provides
implicit memory safety around node allocation not present in the `C` library.

## Getting Started

### Dependencies
* [libcmark](https://github.com/commonmark/cmark#installing)

### Building

Doogie can be built using cargo.

* From the root of the project
    ```
    $ cargo build
    ```

### Installation

Doogie can be integrated into your Rust project by adding it to your
`Cargo.toml` file.

* Add Doogie to `Cargo.toml`
    ```
    [dependencies]
    doogie = { git="https://github.com/PolySync/doogie", branch="devel"}
    ```

## Usage

The basic workflow is to use `parse_document` to parse the textual content of
a Markdown document into the CommonMark AST. You will get a handle to the root
`Document` node with which you can traverse and manipulate the AST. You can
export the document back into a textual form using any of the render methods
such as `Node::render_commonmark()`.

```Rust
use doogie::parse_document;

let document = "# My Great Document \
\
* Item 1 \
* Item 2 \
* Item 3";

let root = parse_document(document);

println!("{}", root.render_xml());
```

### Examples

* Transform all text into uppercase
    ```Rust
    use doogie::{parse_document, Node};

    let document = "# My Great Document \
    \
    * Item 1 \
    * Item 2 \
    * Item 3";

    let root = parse_document(document);

    for (mut node, _) in root.iter() {
        if let Node::Text(ref mut node) = node {
            let content = node.get_content().unwrap();
            node.set_content(&content.to_uppercase()).unwrap();
        }
    }

    ```
* Remove all level 6 `Heading` nodes
    ```Rust
    use doogie::{parse_document, Node};
    
    let document = "# My Great Document \
        \
        * Item 1 \
        * Item 2 \
        * Item 3";
    
    let root = parse_document(document);
    
    for (mut node, _) in root.iter() {
        let prune = match node {
            Node::Heading(ref heading) => heading.get_level() == 6,
            _ => false
        };
    
        if prune {
            node.unlink();
        }
    }
    ```

## Tests

The tests are located inline with the module code in `src/lib.rs`.

### Building

To build jsut the tests run `$ cargo build --tests`.

### Running Tests

The tests are run by invoking `$ cargo test`. This will build them automatically
if necessary.

# License

© 2018, PolySync Technologies, Inc.

* Devin Smith <dsmith@polysync.io>

Please see the [LICENSE](./LICENSE) file for more details
