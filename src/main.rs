mod log;
mod status;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Available commands: status");
        println!("Available options:  --version, --help");
    } else {
        let mode = &args[1];
        if mode == "--version" || mode == "-v" {
            println!("rgt {}", env!("CARGO_PKG_VERSION"));
        } else if mode == "--help" || mode == "-h" {
            println!("Usage: rgt status");
            println!("");
            println!("Options:");
            println!("  -v, --version   Show version");
            println!("  -h, --help      Show help");
        } else if mode == "status" {
            if args.len() >= 3 {
                status::main(args[2].to_string());
            } else {
                status::main("./".to_string());
            }
        } else if mode == "log" {
            if args.len() >= 3 {
                log::main(args[2].to_string());
            } else {
                log::main("./".to_string());
            }
        }
    }
}
