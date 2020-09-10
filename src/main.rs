use walkdir::{DirEntry, WalkDir};
use std::path::Path;
use std::fs;
use termion::color;

fn extract_branch(entry: &DirEntry) -> Option<String> {
    let head_path = format!("{}/HEAD", entry.path().to_string_lossy());

    log::debug!("Checking HEAD for {}", head_path);

    if !Path::new(&head_path).exists() {
        log::debug!("{} does not exist", head_path);
        None
    } else {
        log::debug!("{} does exist", head_path);
        log::debug!("{}", entry.path().to_string_lossy());

        let branch_line = fs::read_to_string(head_path).expect("Something went wrong reading the file");

        let clean_branch = branch_line.trim_end_matches(&['\r', '\n'][..]);
        let branch_parts = clean_branch.split("/");
        let branch_vector: Vec<&str> = branch_parts.collect();
        let branch_name = branch_vector.last().map_or("", |b| b);

        Some(String::from(branch_name))
    }
}

fn main() {
    env_logger::init();

    for entry in WalkDir::new(".")
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();

        if f_name == ".git" {
            let msg = extract_branch(&entry)
                .map(|branch| (entry.path().parent().unwrap().display().to_string(), branch))
                .unwrap_or((String::from(""), String::from("")));

            if msg != (String::from(""), String::from("")) {    
                print!("{}", color::Fg(color::Green));
                print!("{}", msg.0);
                print!("{}", color::Fg(color::Cyan));
                print!("{}", " -> ");
                print!("{}", color::Fg(color::Yellow));
                print!("{}", msg.1);
                print!("{}", "\n");
            }
        }
    }
}
