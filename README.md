<!--
© 2018, Devin Smith <dsmith@polysync.io>

This file is part of Doogie

Doogie is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Doogie is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Doogie.  If not, see <http://www.gnu.org/licenses/>.
-->

# Doogie

## Overview
Doogie is a wrapper library around [cmark](https://github.com/commonmark/cmark), the `C` implementation of 
[CommonMark](http://commonmark.org/). It provides some implicit memory safety around node allocation not 
present in the `C` library.
 
## Getting Started

### Dependencies
* [Install libcmark](https://github.com/commonmark/cmark#installing)

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

### Running Tests

`cargo test`

# License

© 2018, Devin Smith <dsmith@polysync.io>

[GPL version 3](https://github.com/PolySync/doogie/LICENSE)
