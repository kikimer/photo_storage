use std::env;

fn main() {
    let path = env::current_exe().unwrap();
    match sort_files::run(path.parent().unwrap()) {
        Ok(res) => {
            println!("{}",res);
            std::process::exit(0)
        }
        Err(res) => {
            println!("{}",res);
            std::process::exit(-1)
        }
    }
}
