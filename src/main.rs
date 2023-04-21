use std::{fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

use clap::Parser;

/// renls: rename all files in a directory with a list of names from a file or stdin
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to directory with files to be renamed
    #[arg(short, long)]
    path: String,
    /// path to file with new name list (optional if piped through stdin)
    #[arg(short, long, default_value_t = String::new())]
    file: String,
    /// show rename proposal but do not apply
    #[arg(short = 'n', long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,
}

/// Returns vector with new name list
fn get_new_name_list(file_path: &str) -> Vec<String> {
    if !(file_path).is_empty() {
        read_input_stream(File::open(file_path).expect("error: unable to read file."))
    } else  {
        if atty::is(atty::Stream::Stdin) {
            panic!("error: stdin buffer is empty.");
        }
        read_input_stream(io::stdin())
    }
}

/// Returns vector with each line read from provided input stream, ignores empty or comment ('#') lines
fn read_input_stream<R: Read>(input_stream: R) -> Vec<String> {
    BufReader::new(input_stream)
        .lines()
        .filter(|l| !(l.as_ref().unwrap().is_empty() || l.as_ref().unwrap().starts_with('#')))
        .map(|l| l.unwrap().trim().to_string())
        .collect()
}

/// Returns sorted paths of files within provided directory
fn get_file_list(dir_path: &str) -> Vec<PathBuf> {
    let paths = fs::read_dir(dir_path).expect("error: unable to load provided path");
    let mut file_list: Vec<PathBuf> = paths
        .into_iter()
        .map(|p| p.unwrap().path())
        .collect();
    file_list.sort();
    file_list
}

/// Returns map with absolute paths and structure (old_name, new_name)
fn make_rename_pair(new_name_list: Vec<String>, file_list: Vec<PathBuf>) -> HashMap<PathBuf, PathBuf> {
    file_list
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let new_filename = format!("{}{}{}", new_name_list.get(i).unwrap(), ".",
                                       f.extension().unwrap().to_string_lossy());
            let new_filepath = f.parent().unwrap().join(new_filename);
            (f.to_owned(), new_filepath)
        })
        .collect()
}

/// Prints provided map with renaming proposal as (old_name --> new_name)
fn print_rename_proposal(rename_pairs: HashMap<PathBuf, PathBuf>) {
    rename_pairs
        .iter()
        .for_each(|(k, v)| println!("{} --> {}", k.display(), v.display()));
}

/// Applies rename operation defined in provided map with structure (old_name, new_name)
fn rename_files(rename_pairs: HashMap<PathBuf, PathBuf>) {
    rename_pairs
        .iter()
        .for_each(|(k, v)| fs::rename(k, v)
            .expect("error: unable to fulfill renaming operation"));
}

fn main() {
    let args = Args::parse();

    let new_name_list = get_new_name_list(&args.file);
    let ren_file_list = get_file_list(&args.path);
    if new_name_list.len() != ren_file_list.len() {
        println!("error: file list and new name list do not have the same number of items");
        return;
    }

    let rename_pairs = make_rename_pair(new_name_list, ren_file_list);

    if args.dry_run {
        print_rename_proposal(rename_pairs);
    } else {
        rename_files(rename_pairs);
    }
}
