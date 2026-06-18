// ============================================================
//  HSD — Hic Sunt Dracones
//  main.rs — the entry point
//
//  Two modes:
//    hsd <file.hsd>          run the program (interpreter)
//    hsd build <file.hsd>    translate it to C (.c file)
// ============================================================

mod ast;
mod lexer;
mod parser;
mod semantic;
mod interpreter;
mod codegen;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;
use parser::Parser;
use semantic::Analyzer;
use interpreter::Interpreter;
use codegen::CodeGen;
use ast::Program;

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

// Shared front-end: read, lex, parse, analyze. Exits on error.
fn frontend(path: &str) -> Program {
    let source = match fs::read_to_string(path) {
        Ok(text) => text.replace("\r\n", "\n"),
        Err(e) => {
            eprintln!("Cannot read '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexical error: {}", e);
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    };

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
