use walkdir::WalkDir;

fn main() {
    for entry in WalkDir::new(".")
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();

        if f_name == ".git" {
            println!("{}", entry.path().parent().unwrap().display());
        }
    }
}
