use clap::{Parser, Subcommand};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::Metadata;
use std::path::PathBuf;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    /// the absolute path to the directory
    directory_path: String,
}

// the struct for comparing the files for checking duplicates
#[derive(Debug)]
struct FileData {
    pub filepath: PathBuf,
    pub metadata: Metadata,
}

// normalizes files to their basename (without enumrations)
fn normalize_file(filename: &str) -> String {
    // regular expression of format: "filename (number).extension (optional)"
    let regex = Regex::new(r"^(?P<base>.+?)\s*\(\d+\)(?P<ext>\.\w+)?$").unwrap();
    if let Some(capture) = regex.captures(filename) {
        // transform the basename capture to string
        let basename = capture.name("base").unwrap().as_str();
        // handle extension cases as they are optional
        let extension = capture.name("ext").map_or("", |ex| ex.as_str());
        format!("{}{}", basename, extension)
    } else {
        filename.to_string()
    }
}

// filter function for ignoring hidden files
fn is_hidden(dir_entry: &DirEntry) -> bool {
    dir_entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

// compare file data based on their metadata
fn compare_file_data(fd1: &FileData, fd2: &FileData) -> Ordering {
    // creation times of file data
    let c1 = fd1.metadata.created().unwrap();
    let c2 = fd2.metadata.created().unwrap();

    // modified times of file data
    let m1 = fd1.metadata.modified().unwrap();
    let m2 = fd2.metadata.modified().unwrap();


    // sort the paths based on created date (oldest is the first element)
    // ordering is important here (Ascending in this case)
    c1.cmp(&c2)
        // if creation time is equal, sort on modified time
        .then(m1.cmp(&m2))
}

// group all duplicate files into a group
fn group_duplicates(directory: &str) -> HashMap<String, Vec<FileData>> {
    let mut duplicate_map: HashMap<String, Vec<FileData>> = HashMap::new();

    let walker = WalkDir::new(directory).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let e = entry.unwrap();
        // add all same entries into hash map
        let basename = normalize_file(e.path().file_name().unwrap().to_str().unwrap());
        //println!("{:?}", basename);

        // check if basename is already in hash map
        if duplicate_map.contains_key(&basename) {
            // if it does add the path to the hash map
            let pair = duplicate_map.get_mut(&basename).unwrap();
            let file_data = FileData {
                filepath: e.path().to_path_buf(),
                metadata: e.path().metadata().unwrap(),
            };

            pair.push(file_data);
        }
        else {
            // else create a new entry
            let file_data = FileData {
                filepath: e.path().to_path_buf(),
                metadata: e.path().metadata().unwrap(),
            };

            let paths_buff: Vec<FileData> = vec![file_data];
            duplicate_map.insert(basename, paths_buff);
        }
    }

    // only keep the entries with more than one copies
    duplicate_map.retain(|_, v| v.len() > 1);

    
    for values in duplicate_map.values_mut() {
        // sort the vector based on their metadata (creation and modified time, oldest file first)
        values.sort_by(compare_file_data);
    }
    duplicate_map
}

fn delete_duplicates(hashmap: HashMap<String, Vec<FileData>>) {
    for (_k, v) in hashmap {
        // delete all the duplicate files
        for f in v.iter().take(v.len()-1) {
            std::fs::remove_file(&f.filepath).unwrap();
            //println!("{:#?}", f.filepath);
        }
        // rename the most updated file
        let most_updated_file = v.last().unwrap();
        let renamed_file = normalize_file(&v.last().unwrap().filepath.to_str().unwrap());
        std::fs::rename(&most_updated_file.filepath, &renamed_file).unwrap();

    }
}

fn main() {
    // get all the command line arguments
    let args = CLI::parse();

    // read all the files and folders in the directory 
    let path_iter = group_duplicates(&args.directory_path);
    delete_duplicates(path_iter);
}
