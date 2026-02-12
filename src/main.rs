use clap::{Parser, Subcommand};
use codecrafters_git::git;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    CatFile {
        #[arg(short = 'p')]
        pretty: bool,
        object: String,
    },
    HashObject {
        #[arg(short = 'w')]
        write: bool,
        file: String,
    },
}

fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    //eprintln!("Logs from your program will appear here!");

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            git::init();
            println!("Initialized git directory");
        }
        Commands::CatFile { pretty: _, object } => {
            let content = git::cat_file(&object)?;
            print!("{content}");
        }
        Commands::HashObject { write, file, } => {
            let hash = git::hash_object(&file, write)?;
            println!("{hash}");
        }
    }
    Ok(())
}
