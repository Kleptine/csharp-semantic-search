# csharp-semantic-search
An example of a quick Rust CLI to skim CSharp files looking for [Tree Sitter](https://tree-sitter.github.io/tree-sitter/) query patterns. Tree sitter parses the AST, and is great for textual queries (ie. find all classes with field xyz that has modifier abc), but doesn't understand type information or call graphs.

This is not really a general tool. It's more like a bash script. The code is only 100~ lines, and output is not formatted with any meaningful intention. Customize it to your needs! 

Should absolutely support Windows, Linux, and OSX. (But only tested on Windows, 🙇🏻‍♂️)

# Performance

Particular attention was paid towards performance so that the tool can be used interactively! It can scan ~6000  C-Sharp files in a large project in under a second. Queries are executed in parallel across all CPU cores. That said, there is still a lot of low hanging fruit if you need this to be even faster (see todos).

# Usage

First make sure you have Rust and Cargo toolchains installed (see the [one-liner](https://www.rust-lang.org/tools/install)). You will also need to install the default C compilation toolchain for your platform, as tree sitter operates by generating a C-based parser, under the hood. 

Then:

```bash
# Pull this repo, with tree sitter c-sharp submodule.
git clone <this repo>
git submodule update --init --recursive

cargo run --release -- <path_to_dir> "<tree sitter query string>"
```

# Notes:

You can find the Tree Sitter query string documentation [here](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries). You can play around with simpler queries and you'll get a feel for it. You can also instruct Rust to print out the tree sitter AST in S-Expression format (the same format as the query language). To do this:

```rust
my_tree_sitter_node.to_sexp()
```

Keep in mind that Tree Sitter doesn't compile C#, it only parses the AST. You can't query for type information or call graphs, only match structural patterns.

You can also compile this to a static executable via `cargo build --release`.


# Example:

A more complex example. This finds all classes that extend at least one parent, which contain at least one field where the first modifier is not 'public'. (note: this uses powershell string quoting).
```bash
cargo run --release -- D:\Projects\EcsEngine "(class_declaration name: (identifier) bases: (base_list (identifier) @parent) body: (declaration_list (field_declaration . (modifier) @modifier) @field (#not-eq? @modifier ""public"")))"'
```


