//! A simple CLI that matches tree-sitter queries against CSharp files in a directory.
//!
//! Note that this is far from being a "shippable" tool. You should approach this more like
//! a bash script you can tweak to your needs. At the bottom is space for adding custom filtering logic,
//! if you need additional filtering in addition to the tree-sitter query language.

use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    path::PathBuf,
    sync::{atomic::AtomicIsize, Mutex},
};

use rayon::prelude::*;
use tree_sitter::{Language, Query, QueryCapture, QueryCursor};
use walkdir::{WalkDir};

use clap::Parser;

extern "C" {
    fn tree_sitter_c_sharp() -> Language;
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    path: PathBuf,
    pattern: String,
}

// A counter to track total matches found in the searched files.
static TOTAL_COUNT: AtomicIsize = AtomicIsize::new(0);

fn main() {
    // Buffer the lines written to the standard out to avoid lock contention of println!().
    // todo(perf): We make squeeze out some extra performance by replacing this with a parallel append-only data structure.
    let output_text = Mutex::new(Vec::new()); 

    let language = unsafe { tree_sitter_c_sharp() };

    let args = Args::parse();
    let full_pattern = format!("{} @full_pattern_cli_capture", args.pattern); // Add an extra root pattern to force capturing the root pattern for display.

    // The final query built from the user's string.
    let query =
        Query::new(language, &full_pattern).expect("Error building query from given string.");

    // Scan the entire directory *first*, so that we can more easily split the work among worker threads later.
    let mut paths = Vec::new();
    for entry in WalkDir::new(args.path).into_iter() {
        let path = entry.unwrap().into_path();
        let path_str = path.to_str().expect("filename was not valid utf-8");
        if path_str.ends_with(".cs") && !path_str.contains("Test") {
            paths.push(path);
        }
    }

    println!("Searching {} files.", paths.len());

    // Divide the files to be searched into equal portions and send to worker threads, via rayon's par_iter.
    paths.par_iter().for_each(|path| {
        if let Err(e) = parse_file(&path, &query, &output_text) {
            output_text
                .lock()
                .unwrap()
                .push(format!("Skipping [{}] [{}]", path.display(), e));
        }
    });

    for o in output_text.lock().unwrap().iter() {
        println!("{}", o);
    }

    println!(
        "\nFound {} total results.",
        TOTAL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
    );
}

// Parses a single csharp file with Tree Sitter and executes the loaded query against it.
fn parse_file(
    path: &PathBuf,
    query: &Query,
    out: &Mutex<Vec<String>>,
) -> Result<(), anyhow::Error> {
    // Unsafe: Tree Sitter uses a C FFI, so we need unsafe to access the generated C library.
    let language = unsafe { tree_sitter_c_sharp() };

    // todo(perf): We could save a good deal of performance by re-using this parser object.
    // It would need to be a thread local.
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).unwrap();

    // Parse the file.
    let source_code = fs::read_to_string(path)?;
    let tree = parser.parse(&source_code, None).unwrap();

    let mut cursor = QueryCursor::new();
    let root_node = tree.root_node();

    let source_bytes = &*source_code.as_bytes();

    let mut seen_nodes: HashSet<usize> = HashSet::new();

    for m in cursor.matches(query, root_node, source_bytes) {
        let captures: HashMap<_, _> = m
            .captures
            .iter()
            .map(|c: &QueryCapture| (query.capture_names()[c.index as usize].clone(), c))
            .collect();

        // This is the capture we added above so we can access the root node of the query match.
        let full_capture = captures["full_pattern_cli_capture"];

        if seen_nodes.contains(&full_capture.node.id()) {
            continue; // Don't consider at the same node twice. Sometimes the same node can match multiple times.
        }
        seen_nodes.insert(full_capture.node.id());

        TOTAL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        //
        // Custom search filtering can go here:
        //

        // For example, I'm finding only those matches whose '@parent' capture is 'IComponentData'

        // let parent_capture = captures["parent"];
        // let parent = parent_capture.node.utf8_text(source_bytes).unwrap();

        // if parent == "IComponentData" {
        //     let found_text = full_capture.node.utf8_text(source_bytes).unwrap();

        //     let mut output = out.lock().unwrap(); // Lock the output mutex here, so we can write these lines together.
        //     output.push(format!(
        //         "===========================================================\nFound [{}] [{}]\n{}",
        //         path.display(),
        //         parent,
        //         found_text,
        //     ));
        //     for (capture_name, capture) in &captures {
        //         if capture_name == "full_pattern_cli_capture" {
        //             continue;
        //         }
        //         output.push(format!(
        //             "\tCapture [{}] = [{}]",
        //             capture_name,
        //             capture.node.utf8_text(source_bytes).unwrap()
        //         ));
        //     }
        // }


        // For now just print all matches:

        // todo(perf): Formatting moves a lot of data around, we could stream these results instead to get better interactivity. Or return a smaller part of the match.
        let found_text = full_capture.node.utf8_text(source_bytes).unwrap();
        out.lock().unwrap().push(format!(
            "===========================================================\nFound [{}] \n{}",
            path.display(),
            found_text,
        ));
    }
    Ok(())
}
