use std::{env, process};

pub fn args() -> String {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || args[1] != "-c" {
        print_usage();
        process::exit(1);
    }
    args[2].clone()
}

fn print_usage() {
    println!("Usage: exe -c [config_path]]");
    println!("  <config_path>: the configuration file path");
}
