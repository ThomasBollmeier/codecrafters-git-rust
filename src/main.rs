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
        #[arg(short = 'p', help = "Pretty-print the contents of the object")]
        pretty: bool,
        object: String,
    },
    HashObject {
        #[arg(short = 'w', help="Write the object into the git database")]
        write: bool,
        file: String,
    },
    LsTree {
        #[arg(long = "name-only", help = "Only show the file names, not the mode and hash")]
        name_only: bool,
        tree: String,
    },
    WriteTree,
    CommitTree {
        #[arg(short = 'm', help = "Commit message", required = true)]
        message: String,
        #[arg(short = 'p', help = "Parent commit hash")]
        parent: Option<String>,
        tree: String,
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
            let hash = git::bytes_to_hex(&git::hash_object(&file, write)?);
            println!("{hash}");
        }
        Commands::LsTree { tree, name_only } => {
            let output = git::ls_tree(&tree, name_only)?;
            print!("{output}");
        }
        Commands::WriteTree => {
            let hash = git::bytes_to_hex(&git::write_tree(".")?);
            println!("{hash}");
        }
        Commands::CommitTree { message, parent, tree } => {
            let hash = git::bytes_to_hex(&git::commit_tree(&tree, &message, &parent)?);
            println!("{hash}");
        }
    }
    Ok(())
}
