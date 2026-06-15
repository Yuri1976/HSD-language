// ============================================================
//  HSD — Hic Sunt Dracones
//  main.rs — the entry point
//
//  Two modes:
//    hsd <file.hsd>          run the program (interpreter)
//    hsd build <file.hsd>    translate it to C (.c file)
//
//  Phase 9a: module resolution for `affer`.
//  Search order for `affer "name"`:
//    1. <dir of importing file>/name.hsd
//    2. For each path in HSD_PATH: <path>/name.hsd
//  Circular imports are detected and silently skipped.
// ============================================================

mod ast;
mod lexer;
mod parser;
mod semantic;
mod interpreter;
mod codegen;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use lexer::Lexer;
use parser::Parser;
use semantic::Analyzer;
use interpreter::Interpreter;
use codegen::CodeGen;
use ast::{Item, Program};

fn main() {
    let args: Vec<String> = env::args().collect();

    // hsd build <file.hsd>  -> emit C
    if args.len() == 3 && args[1] == "build" {
        let program = frontend(&args[2]);
        let mut cg = CodeGen::new();
        match cg.generate(&program) {
            Ok(c_code) => {
                let out_path = if args[2].ends_with(".hsd") {
                    args[2].replace(".hsd", ".c")
                } else {
                    format!("{}.c", args[2])
                };
                if let Err(e) = fs::write(&out_path, c_code) {
                    eprintln!("Cannot write '{}': {}", out_path, e);
                    process::exit(1);
                }
                println!("Generated {}", out_path);
            }
            Err(e) => {
                eprintln!("Codegen error: {}", e);
                process::exit(1);
            }
        }
        return;
    }

    // hsd <file.hsd>  -> interpret
    if args.len() == 2 {
        let program = frontend(&args[1]);
        let mut interpreter = Interpreter::new();
        if let Err(e) = interpreter.run(&program) {
            eprintln!("Runtime error: {}", e);
            process::exit(1);
        }
        return;
    }

    eprintln!("Usage:");
    eprintln!("  hsd <file.hsd>          run the program");
    eprintln!("  hsd build <file.hsd>    translate it to C");
    process::exit(1);
}

// ---- front-end entry point ----

// Shared front-end: resolve imports, lex, parse, analyze. Exits on error.
fn frontend(path: &str) -> Program {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut items = Vec::new();
    load_file(path, &mut visited, &mut items);

    let program = Program { items };

    let mut analyzer = Analyzer::new();
    if let Err(errors) = analyzer.analyze(&program) {
        eprintln!("Semantic analysis found {} error(s):", errors.len());
        for e in &errors {
            eprintln!("  - {}", e);
        }
        process::exit(1);
    }

    program
}

// ---- recursive module loader ----

// Load one .hsd file, resolve its imports recursively, and append all
// non-Import items to `out`. `visited` tracks canonical paths to detect
// circular imports.
fn load_file(path: &str, visited: &mut HashSet<PathBuf>, out: &mut Vec<Item>) {
    // Canonicalize so that "./foo.hsd" and "foo.hsd" are the same entry.
    let canonical = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Cannot read '{}': {}", path, e);
            process::exit(1);
        }
    };

    // Already loaded — skip (handles circular imports).
    if !visited.insert(canonical.clone()) {
        return;
    }

    let source = match fs::read_to_string(&canonical) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Cannot read '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexical error in '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error in '{}': {}", path, e);
            process::exit(1);
        }
    };

    // The directory of the current file — used to resolve relative imports.
    let file_dir = canonical.parent().unwrap_or(Path::new(".")).to_path_buf();

    for item in program.items {
        match item {
            Item::Import(ref module_name) => {
                // Resolve the module path.
                let resolved = resolve_module(module_name, &file_dir);
                match resolved {
                    Some(mod_path) => {
                        load_file(mod_path.to_str().unwrap(), visited, out);
                    }
                    None => {
                        // Build a helpful error listing all paths searched.
                        let mut searched = vec![
                            file_dir.join(format!("{}.hsd", module_name))
                        ];
                        if let Ok(hsd_path) = env::var("HSD_PATH") {
                            for p in env::split_paths(&hsd_path) {
                                searched.push(p.join(format!("{}.hsd", module_name)));
                            }
                        }
                        eprintln!("Cannot find module '{}' imported in '{}'.", module_name, path);
                        eprintln!("Searched:");
                        for p in &searched {
                            eprintln!("  {}", p.display());
                        }
                        process::exit(1);
                    }
                }
            }
            // All other items go directly into the output.
            other => out.push(other),
        }
    }
}

// ---- module path resolution ----

// Returns the path to the .hsd file for `module_name`, or None if not found.
// Search order:
//   1. <file_dir>/<module_name>.hsd       (relative to importing file)
//   2. <HSD_PATH entry>/<module_name>.hsd (for each entry in HSD_PATH)
fn resolve_module(module_name: &str, file_dir: &Path) -> Option<PathBuf> {
    // Add .hsd extension if not already present.
    let file_name = if module_name.ends_with(".hsd") {
        module_name.to_string()
    } else {
        format!("{}.hsd", module_name)
    };

    // 1. Relative to the importing file.
    let relative = file_dir.join(&file_name);
    if relative.exists() {
        return Some(relative);
    }

    // 2. HSD_PATH entries.
    if let Ok(hsd_path) = env::var("HSD_PATH") {
        for search_dir in env::split_paths(&hsd_path) {
            let candidate = search_dir.join(&file_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}
