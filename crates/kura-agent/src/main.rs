fn main() {
    if let Err(e) = kura_agent::run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
