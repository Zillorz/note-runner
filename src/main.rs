use std::{
    fs::File,
    io::{Read, Write, stdin, stdout},
    process::Command,
};

use anyhow::{Result, bail};
use markdown::{
    ParseOptions,
    mdast::{Code, Node},
};
use tempdir::TempDir;

// nr <file> 1
// nr <file> l1
//
// nr repl <file>
// IN REPL:
// exit -> quit
// l1 -> runs at line 1
// 2 -> runs 2nd block

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        bail!("Usage: note-runner <repl> <filename> OR note-runner <filemame> <block number>");
    }

    if args.len() < 3 {
        bail!("Not enough arguments...");
    }

    let file_name = if args[1] == "repl" {
        &args[2]
    } else {
        &args[1]
    };

    if args[1] == "repl" {
        let mut line = String::new();

        loop {
            print!("> ");
            line.clear();
            let _ = stdout().flush();
            stdin().read_line(&mut line)?;

            if line.trim() == "exit" {
                break;
            } else {
                let res = run_block(&load_file(file_name)?, line.trim());

                if let Err(e) = res {
                    eprintln!("{e}");
                }
            }
        }
    } else {
        run_block(&load_file(file_name)?, &args[2])?;
    }

    Ok(())
}

fn run_block(blocks: &[Code], wblock: &str) -> Result<()> {
    if wblock.starts_with("l") {
        let line = wblock.trim_start_matches("l").parse::<usize>()? - 1;

        for (idx, block) in blocks.iter().enumerate() {
            let pos = block.position.clone().unwrap();

            if pos.start.line <= line && line < pos.end.line {
                run_code(blocks, idx)?;
                break;
            }
        }
    } else {
        let idx: usize = wblock.parse::<usize>()? - 1;

        if idx >= blocks.len() {
            println!("There are only {} blocks!", blocks.len());
        } else {
            run_code(&blocks, idx)?;
        }
    }

    Ok(())
}

fn run_code(blocks: &[Code], run: usize) -> Result<()> {
    let tmp_dir = TempDir::new("note-code-runner")?;
    let file_dir = tmp_dir.path().join("code.c");

    let mut tmp_file = File::create(file_dir)?;

    writeln!(tmp_file, "#include <stdio.h>")?;
    writeln!(tmp_file, "#include <string.h>")?;
    writeln!(tmp_file, "#include <stdlib.h>")?;

    let block = &blocks[run];

    if let Some(meta) = &block.meta
        && meta == "standalone"
    {
        tmp_file.write_all(block.value.as_bytes())?;
    } else {
        // let's add the libraries
        for lib in blocks
            .iter()
            .take(run) // only add libraries BEFORE
            .filter(|x| x.meta.as_ref().is_some_and(|x| x == "lib"))
        {
            writeln!(tmp_file)?;
            tmp_file.write_all(lib.value.as_bytes())?;
            writeln!(tmp_file)?;
        }

        writeln!(tmp_file)?;
        writeln!(tmp_file, "int main() {{")?;
        tmp_file.write_all(block.value.as_bytes())?;
        writeln!(tmp_file)?;
        writeln!(tmp_file, "return 0; }}")?;
    }

    let cmd = Command::new("gcc")
        .current_dir(&tmp_dir)
        .arg("code.c")
        .status()?;

    if !cmd.success() {
        bail!("Failed to compile the program!");
    }

    Command::new(tmp_dir.path().join(if cfg!(windows) { "a.exe" } else { "a.out" })).spawn()?.wait()?;
    println!();

    Ok(())
}

fn load_file(file_name: &str) -> Result<Vec<Code>> {
    let mut file = File::open(file_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mast = markdown::to_mdast(&content, &ParseOptions::default()).unwrap();

    let mut code_blocks = Vec::new();
    process_nodes(&mast, &mut code_blocks);
    Ok(code_blocks)
}

fn process_nodes(n: &Node, collection: &mut Vec<Code>) {
    match n {
        Node::Code(code) => {
            if let Some(lang) = &code.lang
                && lang == "c"
            {
                collection.push(code.clone());
            }
        }
        n => {
            if let Some(children) = n.children() {
                for child in children {
                    process_nodes(child, collection);
                }
            }
        }
    }
}
