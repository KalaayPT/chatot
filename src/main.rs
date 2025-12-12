use std::path::PathBuf;

use clap::{Parser, Subcommand, Args, CommandFactory};
use clap::error::ErrorKind;
mod decode;
mod charmap;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Decrypt and decode binary text archive to text files
    Decode {
        /// Path to custom character map file
        #[arg(short='m', long)]
        charmap: PathBuf,
        #[command(flatten)]
        source: BinarySource,
        #[command(flatten)]
        destination: TextSource,
        
    },
    /// Encrypt and encode text files to binary text archive
    Encode {
        /// Path to custom character map file
        #[arg(short='m', long)]
        charmap: PathBuf,
        #[command(flatten)]
        source: TextSource,
        #[command(flatten)]
        destination: BinarySource,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct BinarySource {
    /// Path(s) to the binary text archive(s)        
    #[arg(short, long, num_args = 1.., conflicts_with = "archive_dir")]
    archive: Option<Vec<std::path::PathBuf>>,
    /// Directory for archives
    #[arg(short='s', long, conflicts_with = "archive")]
    archive_dir: Option<std::path::PathBuf>,
    
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct TextSource {
    /// Path(s) to the text file(s)
    #[arg(short, long, num_args = 1.., conflicts_with = "text_dir")]
    txt: Option<Vec<std::path::PathBuf>>,
    /// Directory for text files
    #[arg(short='d', long, conflicts_with = "txt")]
    text_dir: Option<std::path::PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Decode {charmap, source, destination } => {
            // Ensure input isn't a directory when output is files
            if source.archive_dir.is_some() && destination.txt.is_some() {
                let mut cmd = Cli::command();
                cmd.error(ErrorKind::ArgumentConflict,
                "Cannot use archive directory with text file outputs",
            )
            .exit();
            }

            let charmap = charmap::read_charmap(charmap)?;

            //println!("{}", charmap.decode_map.get(&0x01DE).unwrap());

            decode::decode_archives(&charmap, source, destination)
        }
        Commands::Encode { .. } => {
            // Placeholder for encode functionality
            Ok(())
        }
    }
}