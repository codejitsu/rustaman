use walkdir::{DirEntry, WalkDir};
use termion::color;
use git2::Repository;
use git2::StatusOptions;
use git2::Error;
use git2::ErrorCode;
use std::fmt;

pub struct RepoStats {
    modified:       u32,
    new:            u32,
    deleted:        u32,
    renamed:        u32,
    typechanged:    u32,
    ignored:        u32
}

impl RepoStats {
    fn add_modified(&mut self) {
        self.modified +=1;
    }

    fn add_new(&mut self) {
        self.new +=1;
    }
    
    fn add_deleted(&mut self) {
        self.deleted +=1;
    }
    
    fn add_renamed(&mut self) {
        self.renamed +=1;
    }
    
    fn add_typechanged(&mut self) {
        self.typechanged +=1;
    }    

    fn add_ignored(&mut self) {
        self.ignored +=1;
    }        
}

impl fmt::Display for RepoStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "m: {}, n: {}, d: {}, r: {}, t: {}, i: {}", self.modified, self.new, self.deleted, self.renamed, self.typechanged, self.ignored)
    }
}

fn make_repo_description(entry: &DirEntry) -> Option<String> {
    let repo = match Repository::open(entry.path()) {
        Ok(r) => r,
        Err(_e) => return Some(format!("failed to open: {}", entry.path().display()))
    }; 

    if repo.is_bare() {
        return Some(String::from("cannot report status on bare repository"));
    } else {
        let mut opts = StatusOptions::new();

        opts.include_untracked(true).recurse_untracked_dirs(true);

        let statuses = match repo.statuses(Some(&mut opts)) {
            Ok(st) => st,
            Err(e) => return Some(format!("failed to fetch status: {}", e))
        };

        let branch = match get_branch(&repo) {
            Ok(b) => Some(b),
            Err(_e) => None
        };

        let repo_stats = get_stats(&statuses);

        return branch.map(|b| format!("{} {}", b, repo_stats));
    }
}

fn get_branch(repo: &Repository) -> Result<String, Error> {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => return Err(e),
    };
    let head = head.as_ref().and_then(|h| h.shorthand());

    Ok(format!("{}", head.unwrap_or("HEAD (no branch)")))
}

fn get_stats(statuses: &git2::Statuses) -> RepoStats {
    let mut repo_stats = RepoStats {
        modified:       0,
        new:            0,
        deleted:        0,
        renamed:        0,
        typechanged:    0,
        ignored:        0
    };

    // Print index changes
    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let _istatus = match entry.status() {
            s if s.contains(git2::Status::INDEX_NEW) => {
                repo_stats.add_new();
            },
            s if s.contains(git2::Status::INDEX_MODIFIED) => {
                repo_stats.add_modified();
            },
            s if s.contains(git2::Status::INDEX_DELETED) => {
                repo_stats.add_deleted();
            },
            s if s.contains(git2::Status::INDEX_RENAMED) => {
                repo_stats.add_renamed();
            },
            s if s.contains(git2::Status::INDEX_TYPECHANGE) => {
                repo_stats.add_typechanged();
            },
            _ => continue,
        };
    }

    // Print workdir changes to tracked files
    for entry in statuses.iter() {
        // With `Status::OPT_INCLUDE_UNMODIFIED` (not used in this example)
        // `index_to_workdir` may not be `None` even if there are no differences,
        // in which case it will be a `Delta::Unmodified`.
        if entry.status() == git2::Status::CURRENT || entry.index_to_workdir().is_none() {
            continue;
        }

        let _istatus = match entry.status() {
            s if s.contains(git2::Status::WT_MODIFIED) => {
                repo_stats.add_modified();
            },
            s if s.contains(git2::Status::WT_DELETED) => {
                repo_stats.add_deleted();
            },
            s if s.contains(git2::Status::WT_RENAMED) => {
                repo_stats.add_renamed();
            },
            s if s.contains(git2::Status::WT_TYPECHANGE) => {
                repo_stats.add_typechanged();
            },
            _ => continue,
        };
    }

    // Print untracked files
    for _entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::WT_NEW)
    {
        repo_stats.add_new();
    }

    // Print ignored files
    for _entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::IGNORED)
    {
        repo_stats.add_ignored();
    }

    return repo_stats;
}

fn main() {
    env_logger::init();

    for entry in WalkDir::new(".")
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();

        if f_name == ".git" {
            let msg = make_repo_description(&entry)
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
