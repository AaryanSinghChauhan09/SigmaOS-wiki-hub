//! CLI for sigmaos-wiki-hub.

use clap::{Parser, Subcommand};
use sigmaos_wiki_hub::{default_index_path, WikiIndex};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sigmaos-wiki-hub", about = "SigmaOS wiki documentation hub")]
struct Cli {
    /// Wiki root directory to index.
    #[arg(long, global = true, default_value = ".")]
    root: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build index and print stats.
    Build,
    /// Search the index.
    Search { query: String },
    /// Print table of contents.
    Toc,
    /// Print a page's content by title.
    Show { title: String },
    /// Save index to JSON.
    Save {
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Load and query a saved index.
    Load { file: PathBuf },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Build => {
            let idx = WikiIndex::build(&cli.root)?;
            println!(
                "indexed {} pages in {} categories",
                idx.pages.len(),
                idx.by_category().len()
            );
        }
        Commands::Search { query } => {
            let idx = WikiIndex::build(&cli.root)?;
            for p in idx.search(&query) {
                println!("[{}] {} — {}", p.category, p.title, p.path.display());
            }
        }
        Commands::Toc => {
            let idx = WikiIndex::build(&cli.root)?;
            print!("{}", idx.toc());
        }
        Commands::Show { title } => {
            let idx = WikiIndex::build(&cli.root)?;
            match idx.get(&title) {
                Some(p) => println!("{}\n\n{}", p.path.display(), p.content),
                None => {
                    // fuzzy: first title containing query
                    let hits = idx.search(&title);
                    if let Some(p) = hits.first() {
                        println!("{}\n\n{}", p.path.display(), p.content);
                    } else {
                        return Err(format!("no page matching '{title}'").into());
                    }
                }
            }
        }
        Commands::Save { file } => {
            let idx = WikiIndex::build(&cli.root)?;
            let path = file.or_else(default_index_path).ok_or("no output path")?;
            idx.save(&path)?;
            println!("saved: {}", path.display());
        }
        Commands::Load { file } => {
            let idx = WikiIndex::load(&file)?;
            println!("loaded {} pages", idx.pages.len());
        }
    }
    Ok(())
}
