use std::env;

fn main() {
    let path = env::current_exe().unwrap();
    let command = env::args().nth(1).unwrap_or_default();
    match sort_files::run(path.parent().unwrap(), command) {
        Ok(_) => std::process::exit(0),
        Err(res) => {
            eprintln!("{}",res);
            std::process::exit(-1)
        }
    }
}
