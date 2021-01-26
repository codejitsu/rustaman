#[macro_use] extern crate log;

use walkdir::{DirEntry, WalkDir};
use termion::color;
use git2::{Repository, StatusOptions, Error, ErrorCode};
use std::fmt;
use console::Emoji;
use structopt::StructOpt;
use std::path::PathBuf;
use std::env;
use log::Level;
use std::time::Instant;

static DONE: Emoji<'_, '_> = Emoji("😇 ", ":-)");

#[derive(Debug, StructOpt)]
struct Opts {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Start directory
    #[structopt(short, long, default_value = ".")]
    root: PathBuf,

    /// branch names to ignore
    #[structopt(short, long)]    
    ignore_list: Vec<String>
}

pub struct RepoStats {
    modified:       u32,
    new:            u32,
    deleted:        u32,
    renamed:        u32,
    typechanged:    u32,
    ignored:        u32,
    ahead:          bool,
    behind:         bool
}

impl RepoStats {
    fn new() -> RepoStats {
        return RepoStats {
            modified:       0,
            new:            0,
            deleted:        0,
            renamed:        0,
            typechanged:    0,
            ignored:        0,
            ahead:          false,
            behind:         false
        };
    }

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

    fn set_ahead(&mut self) {
        self.ahead = true;
    }

    fn set_behind(&mut self) {
        self.behind = true;
    }    
}

impl fmt::Display for RepoStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.modified + self.new + self.deleted + self.renamed + self.typechanged + self.ignored == 0 {
            write!(f, "{}{}", color::Fg(color::Green), " ✔ ")?;
        } 
        
        if self.ahead {
            write!(f, "{}{}", color::Fg(color::Magenta), " ↑ ")?;
        }

        if self.behind {
            write!(f, "{}{}", color::Fg(color::Cyan), " ↓ ")?;
        }

        if self.modified > 0 {
            write!(f, "{}{}{}", color::Fg(color::Blue), " ✹ ", self.modified)?;
        }
        
        if self.new > 0 {
            write!(f, "{}{}{}", color::Fg(color::Green), " ✚ ", self.new)?;            
        }
        
        if self.deleted > 0 {
            write!(f, "{}{}{}", color::Fg(color::Red), " ✖ ", self.deleted)?;                        
        }
        
        if self.renamed > 0 {
            write!(f, "{}{}{}", color::Fg(color::White), " ➜ ", self.renamed)?;                                    
        } 
        
        Ok(())
    }
}

fn make_repo_description(entry: &DirEntry, opts: &Opts) -> Result<String, String> {
    let repo = match Repository::open(entry.path()) {
        Ok(r) => r,
        Err(_e) => return Ok(format!("failed to open: {}", entry.path().display()))
    }; 

    if repo.is_bare() {
        return Ok(String::from("cannot report status on bare repository"));
    } else {
        let mut status_opts = StatusOptions::new();

        status_opts.include_untracked(true).recurse_untracked_dirs(true);

        let statuses = match repo.statuses(Some(&mut status_opts)) {
            Ok(st) => st,
            Err(e) => return Ok(format!("failed to fetch status: {}", e))
        };

        let branch = match get_branch(&repo) {
            Ok(name) => name,
            Err(e) => return Err(e.to_string())
        };

        let repo_stats = get_stats(&statuses, &repo);

        let branch_info = if !opts.ignore_list.is_empty() && !opts.ignore_list.contains(&branch) {
            format!("{}{}", color::Fg(color::Red), branch)
        } else {
            format!("{}{}", color::Fg(color::Yellow), branch)
        };

        return Ok(format!("{} {}", branch_info, repo_stats));
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

fn get_stats(statuses: &git2::Statuses, repo: &Repository) -> RepoStats {
    let mut repo_stats = RepoStats::new();

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

    let (ahead, behind) = is_ahead_behind_remote(repo);

    if ahead {
        repo_stats.set_ahead();
    }

    if behind {
        repo_stats.set_behind();
    }

    return repo_stats;
}

/// Determine if the current HEAD is ahead/behind its remote. The tuple
/// returned will be in the order ahead and then behind.
///
/// If the remote is not set or doesn't exist (like a detached HEAD),
/// (false, false) will be returned.
fn is_ahead_behind_remote(repo: &Repository) -> (bool, bool) {
    let head_res = repo.revparse_single("HEAD");

    let head = match head_res {
        Ok(h) => h.id(),
        Err(_) => return (false, false)
    };

    if let Some((upstream, _)) = repo.revparse_ext("@{u}").ok() {
        return match repo.graph_ahead_behind(head, upstream.id()) {
            Ok((commits_ahead, commits_behind)) => (commits_ahead > 0, commits_behind > 0),
            Err(_) => (false, false),
        };
    }
    (false, false)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
    .to_str()
    .map(|s| s.starts_with("."))
    .unwrap_or(false)
}

fn run(opts: &Opts) -> Result<(), String> {
    let start = Instant::now();

    if env::var("RUST_LOG").is_err() && opts.debug {
        env::set_var("RUST_LOG", "debug")
    } else {
        env::set_var("RUST_LOG", "info")
    }

    env_logger::init();
    
    debug!("Using command line parameters: {:?}", opts);

    for entry in WalkDir::new(opts.root.to_str().unwrap_or("."))
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !e.path().is_file() && is_hidden(e))
            .filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();

        debug!("Processing path: {}", entry.path().display());

        if f_name == ".git" {
            let msg = make_repo_description(&entry, &opts)
                .map(|repo_info| (entry.path().parent().unwrap().display().to_string(), repo_info));

            match msg {
                Ok((path, description)) => {
                    print!("{}{}", color::Fg(color::Green), path);
                    print!("{}{}", color::Fg(color::Cyan), " -> ");
                    print!("{}{}\n", color::Fg(color::Yellow), description);                 
                }

                Err(_e) => continue
            }
        }
    }

    let duration = start.elapsed();

    let duration_str = if duration.as_secs() == 0 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{}s", duration.as_secs())
    };

    log!(Level::Info, "{}{} Done! in {}", color::Fg(color::White), DONE, duration_str);

    return Ok(());
}

fn main() {
    let opts = Opts::from_args();
    match run(&opts) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}