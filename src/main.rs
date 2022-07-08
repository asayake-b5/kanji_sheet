use clap::{Parser, Subcommand};
use kanji_practice_sheet::{arg_parsing::kanji_to_filename, pdf_creation::kanji_to_png};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the webserver
    Server {},
    /// Use the generator directly
    #[clap(arg_required_else_help = true)]
    Cli {
        //TODO interactive mode?
        kanjis: String,
    },
}
fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Server {} => {}
        Commands::Cli { kanjis } => {
            // if let Some(k) = kanjis.chars().nth(0) {
            let now = std::time::Instant::now();
            string_to_png(&kanjis);
            // kanji_to_png(&kanji_to_filename(k));
            println!("{:?}", now.elapsed());
            // }
        }
    };
}
