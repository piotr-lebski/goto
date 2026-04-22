use clap::Parser;

fn main() {
    let cli = goto::cli::Cli::parse();

    if let Err(error) = goto::app::run(cli) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
