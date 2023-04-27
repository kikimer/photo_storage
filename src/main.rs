use std::env;

use clap::Parser;

/// Manage files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Check storage
    #[arg(short, long)]
    check: bool,

    /// create new index by storage files
    #[arg(short, long)]
    reindex: bool,

    /// path to index folder
    #[arg(short, long, default_value_t = String::new())]
    path: String,
}

fn main() {
    let args = Args::parse();
    let path = env::current_dir().unwrap().join(args.path);
    println!("use path: {path:?}");
    let result = if args.check {
        sort_files::check_stored_files(&path)
    } else if args.reindex {
        sort_files::rebuild_index(&path)
    } else {
        sort_files::add_new_files(&path)
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(res) => {
            eprintln!("{}", res);
            std::process::exit(-1)
        }
    };
}