use clap::Parser;
use std::path::PathBuf;
use std::process;
use grobr::finder::find_files;
use grobr::grouper::group_files;
use grobr::parser::parse_declaration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The pattern declaration string
    declaration: String,

    /// Input directory to search
    #[arg(value_name = "DIR")]
    directory: PathBuf,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let declaration = parse_declaration(&cli.declaration)?;
    let collections = find_files(&cli.directory, declaration)?;
    let groups = group_files(collections);

    serde_json::to_writer(std::io::stdout(), &groups)?;

    Ok(())
}
