use clap::{Parser, Subcommand, Args};
mod decode;

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
        #[command(flatten)]
        source: BinarySource,
        #[command(flatten)]
        destination: TextSource,
    },
    /// Encrypt and encode text files to binary text archive
    Encode {
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
        Commands::Decode { source, destination } => {
            decode::decode_archives(source, destination)
        }
        Commands::Encode { .. } => {
            // Placeholder for encode functionality
            Ok(())
        }
    }
}