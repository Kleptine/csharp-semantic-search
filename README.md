# csharp-semantic-grep
An example of a quick Rust CLI to skim CSharp files looking for [Tree Sitter](https://tree-sitter.github.io/tree-sitter/) query patterns.

This is not really a general tool. It's more like a bash script. The code is only 100~ lines, and output is not formatted with any meaningful intention. Customize it to your needs! 

# Performance

Particular attention was paid towards performance so that the tool can be used interactively! It can scan ~6000 large C-Sharp files in under a second. Queries are executed in parallel across all CPU cores. 

# Usage

First make sure you have Rust and Cargo toolchains installed (see the [one-liner](https://www.rust-lang.org/tools/install)). You will also need to install the default C compilation toolchain for your platform, as tree sitter operates by generating a C-based parser, under the hood. That said, there is still a lot of low hanging fruit if you need this to be even faster (see todos).

Then:

`cargo run --release -- <path_to_dir> "<tree sitter query string>"`

# Notes:

You can find the Tree Sitter query string documentation [here](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries). You can play around with simpler queries and you'll get a feel for it. You can also instruct Rust to print out the tree sitter AST in S-Expression format (the same format as the query language). To do this:

```rust
my_tree_sitter_node.to_sexp()
```

You can also compile this to a static executable via `cargo build --release`.

Should absolutely support Windows, Linux, and OSX. (But only tested on Windows, 🙇🏻‍♂️)
```

# Example:
`cargo run --release -- D:\Projects\EcsEngine "(class_declaration name: (identifier) bases: (base_list (identifier) @parent) body: (declaration_list (field_declaration . (modifier) @modifier) @field (#not-eq? @modifier ""public"")))"'`
