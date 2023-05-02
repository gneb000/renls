use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::{fs, io};
use std::process::exit;

use clap::Parser;

/// renls: rename all files in a directory with a list of names from a file or stdin
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to directory with files to be renamed
    #[arg(value_name = "DIR_PATH")]
    path: String,
    /// path to file with new name list (optional if piped through stdin)
    #[arg(short, long, default_value_t = String::new())]
    file: String,
    /// show rename proposal but do not apply
    #[arg(short = 'n', long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,
}

/// Returns vector with new name list
fn get_new_name_list(file_path: &str) -> Result<Vec<String>, &str> {
    if (file_path).is_empty() {
         if atty::isnt(atty::Stream::Stdin) {
             Ok(read_input_stream(io::stdin()))
         } else {
             Err("renls: error: stdin buffer is empty")
         }
    } else {
        match File::open(file_path) {
            Ok(file) => Ok(read_input_stream(file)),
            Err(_) => Err("renls: error: unable to read file"),
        }
    }
}

/// Returns vector with each line read from provided input stream, ignores empty or comment ('#') lines
fn read_input_stream<R: Read>(input_stream: R) -> Vec<String> {
    BufReader::new(input_stream)
        .lines()
        .filter(|l| l.is_ok() && !(l.as_ref().unwrap().is_empty() || l.as_ref().unwrap().starts_with('#')))
        .map(|l| l.unwrap().trim().to_string())
        .collect()
}

/// Returns sorted paths of files within provided directory
fn get_file_list(dir_path: &str) -> Result<Vec<PathBuf>, &str>{
    match fs::read_dir(dir_path) {
        Ok(paths) => {
            let mut file_list: Vec<PathBuf> = paths
                .into_iter()
                .filter(|p| Result::is_ok(p))
                .map(|p| p.unwrap().path())
                .collect();
            file_list.sort();
            Ok(file_list)
        },
        Err(_) => Err("renls: error: unable to read directory"),
    }
}

/// Returns map with absolute paths and structure (`old_name`, `new_name`)
fn make_rename_pair(new_name_list: &[String], file_list: &[PathBuf]) -> HashMap<PathBuf, PathBuf> {
    file_list
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ext_str = match f.extension() {
                Some(ext) => format!(".{}", ext.to_str().unwrap_or("")),
                None => String::new(),
            };
            let new_filename = format!("{}{}", &new_name_list[i], ext_str);
            let new_filepath = f.parent().unwrap().join(new_filename);
            (f.clone(), new_filepath)
        })
        .collect()
}

/// Prints provided map with renaming proposal as (`old_name` --> `new_name`)
fn print_rename_proposal(rename_pairs: &HashMap<PathBuf, PathBuf>) {
    for (k, v) in rename_pairs.iter() {
        println!("{} --> {}", k.display(), v.display());
    }
}

/// Applies rename operation defined in provided map with structure (`old_name`, `new_name`)
fn rename_files(rename_pairs: &HashMap<PathBuf, PathBuf>) {
    for (k, v) in rename_pairs {
        if fs::rename(k, v).is_err() {
            println!("renls: error: unable to rename file \"{}\"", k.display());
        }
    }
}

fn main() {
    let args = Args::parse();

    let new_name_list = match get_new_name_list(&args.file) {
        Ok(list) => list,
        Err(error_message) => {
            println!("{error_message}");
            exit(1);
        }
    };
    let ren_file_list = match get_file_list(&args.path) {
        Ok(list) => list,
        Err(error_message) => {
            println!("{error_message}");
            exit(1);
        }
    };
    if new_name_list.len() != ren_file_list.len() {
        println!("renls: error: file list and new name list do not have the same number of items");
        exit(1);
    }

    let rename_pairs = make_rename_pair(&new_name_list, &ren_file_list);

    if args.dry_run {
        print_rename_proposal(&rename_pairs);
    } else {
        rename_files(&rename_pairs);
    }
}
