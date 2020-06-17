mod status;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Available commands: status, version")
    } else {
        let mode = &args[1];
        if mode == "version" {
            println!("rgt {}", env!("CARGO_PKG_VERSION"))
        } else if mode == "status" {
            if args.len() >= 3 {
                status::main(args[2].to_string());
            } else {
                status::main("./".to_string());
            }
        }
    }
}
