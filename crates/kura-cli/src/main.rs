#[tokio::main]
async fn main() {
    std::process::exit(kura_cli::run_from_args(std::env::args()).await);
}
