use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use clap::Parser;

fn txt_ren(dir_path: &str, text_file_path: &str, dry_run: bool) {
    let new_name_list = load_text_file_content(text_file_path);
    let ren_file_list = get_file_list(dir_path);

    if new_name_list.len() != ren_file_list.len() {
        println!("Error: File list and new name list do not have the same number of items.");
        return;
    }

    let rename_pairs = make_rename_pair(new_name_list, ren_file_list);

    if dry_run {
        print_rename_proposal(rename_pairs);
    } else {
        rename_files(rename_pairs);
    }
}

fn load_text_file_content(file_path: &str) -> Vec<String> {
    let file = fs::File::open(file_path).expect("Error: Unable to read file.");
    BufReader::new(file)
        .lines()
        .filter(|l| !(l.as_ref().unwrap().is_empty() || l.as_ref().unwrap().starts_with('#')))
        .map(|l| l.unwrap().trim().to_string())
        .collect()
}

fn get_file_list(dir_path: &str) -> Vec<String> {
    let paths = fs::read_dir(dir_path).expect("Error: Unable to load provided path.");
    let mut file_list: Vec<String> = paths
        .into_iter()
        .map(|p| p.unwrap().path().to_str().unwrap().to_string())
        .collect();
    file_list.sort();
    file_list
}

fn make_rename_pair(new_name_list: Vec<String>, file_list: Vec<String>) -> HashMap<String, String> {
    let mut rename_pairs = HashMap::new();
    file_list
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let file_path = Path::new(n);
            let file_ext = ".".to_owned() + file_path.extension().unwrap().to_str().unwrap();
            let parent_dir = file_path.parent().unwrap();
            let new_name= new_name_list.get(i).unwrap().to_owned() + &file_ext;
            let new_path = parent_dir.join(new_name).to_str().unwrap().to_string();
            rename_pairs.insert(n.clone(), new_path);
        })
        .count();
    rename_pairs
}

fn print_rename_proposal(rename_pairs: HashMap<String, String>) {
    rename_pairs
        .iter()
        .map(|(k, v)| println!("{} --> {}", k, v))
        .count();
}

fn rename_files(rename_pairs: HashMap<String, String>) {
    rename_pairs
        .iter()
        .map(|(k, v)| fs::rename(k, v))
        .count();
}

/// txtren: rename all files in a directory with a list of names in a text file
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path: path to directory with files to be renamed
    #[arg(short, long)]
    path: String,
    /// file: path to file with new name list
    #[arg(short, long)]
    file: String,
    /// dry_run: show rename proposal but do not apply
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,
}

fn main() {
    let args = Args::parse();
    txt_ren(&args.path, &args.file, args.dry_run);
}
