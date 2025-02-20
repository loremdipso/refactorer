use log::{info, LevelFilter};
use rand::rng;
use rand::seq::SliceRandom;
use regex::Regex;
use simple_logger::SimpleLogger;
use std::{fs::read_to_string, path::Path, process::Command};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Download stuff")]
struct Opt {
    /// Increase verbosity
    #[structopt(short, long)]
    verbose: bool,

    /// Custom cache filename
    #[structopt(short, long)]
    cache_filename: Option<String>,

    /// Which extensions to search for
    #[structopt(short, long)]
    extensions: Option<String>,

    /// Filter for files.
    #[structopt(short, long)]
    filter: Option<String>,

    /// Go smallest to largest
    #[structopt(long)]
    smallest: bool,

    /// Go largest to smallest
    #[structopt(long)]
    largest: bool,

    /// Program to use to edit files
    #[structopt(long)]
    program: Option<String>,

    /// Program to use to edit files
    #[structopt(long)]
    args: Vec<String>,
}

fn main() {
    let options = Opt::from_args();
    let logger = SimpleLogger::new().with_colors(true).without_timestamps();
    if options.verbose {
        logger.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        logger.with_level(LevelFilter::Info).init().unwrap();
    }

    let filename = options
        .cache_filename
        .clone()
        .unwrap_or(".refactorer.cache".into());
    let mut unseen_files = get_files(&options);
    let mut refactored_files: Vec<String> = Vec::new();
    if let Ok(existent_files) = read_to_string(&filename) {
        info!("Reading from cache file...");
        // TODO: this is bad perf
        for line in existent_files.split("\n") {
            refactored_files.push(line.into());
            unseen_files.retain(|l| l != line);
        }
    }

    if options.smallest {
        unseen_files.sort_by_cached_key(|path| get_size(path));
    } else if options.largest {
        unseen_files.sort_by_cached_key(|path| -get_size(path));
    } else {
        unseen_files.shuffle(&mut rng());
    }

    let mut index = 0;
    loop {
        if index >= unseen_files.len() {
            break;
        }

        let path = &unseen_files[index];
        println!(
            "\n\nEditing file ({}/{}): {}",
            index + 1,
            unseen_files.len(),
            path
        );
        edit_file(path, &options);
        let input = readline("Sufficiently refactored? (y/n/q): ");
        match input.chars().next() {
            Some(c) => match c {
                'y' => {
                    refactored_files.push(path.into());
                    info!("Writing to cache file...");
                    std::fs::write(&filename, refactored_files.join("\n")).expect("");
                }
                'n' => {
                    // do nothing
                }
                'q' => {
                    break;
                }
                _ => {
                    break;
                }
            },
            None => {
                break;
            }
        }
        index += 1;
    }
}

fn edit_file(path: &str, options: &Opt) {
    let exe = match &options.program {
        Some(program) => program,
        None => &"code".into(),
    };

    Command::new(exe)
        .args(&options.args)
        .arg(path)
        .spawn()
        .unwrap();
}

fn get_files(options: &Opt) -> Vec<String> {
    let extensions = match &options.extensions {
        Some(str) => str.split(",").map(|s| s.to_owned()).collect(),
        None => vec![".rs".into()],
    };

    let maybe_regex = match &options.filter {
        Some(filter) => Some(Regex::new(filter).unwrap()),
        None => None,
    };

    let valid = |path: &Path| -> bool {
        let filename = path.to_string_lossy();
        return extensions.iter().any(|ext| filename.ends_with(ext))
            && match &maybe_regex {
                Some(regex) => regex.is_match(&filename),
                None => true,
            };
    };

    walkdir::WalkDir::new(".")
        .into_iter()
        .filter_entry(|entry| {
            if let Some(file_name) = entry.file_name().to_str() {
                return !is_hidden(file_name);
            }
            return true;
        })
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            _ => None,
        })
        .filter(|entry| !entry.path().is_dir() && valid(entry.path()))
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect()
}

fn is_hidden(file_name: &str) -> bool {
    // True for hidden files/dirs, but false for the current directory.
    return file_name.starts_with(".") && file_name.len() > 1;
}

fn readline(prompt: &str) -> String {
    println!("{prompt}");
    let mut s = String::new();
    _ = std::io::stdin().read_line(&mut s);
    s
}

fn get_size(path: &str) -> i64 {
    match std::fs::File::open(path) {
        Ok(f) => match f.metadata() {
            Ok(metadata) => metadata.len() as i64,
            Err(_) => 0,
        },
        Err(_) => 0,
    }
}
