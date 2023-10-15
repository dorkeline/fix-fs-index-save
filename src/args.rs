use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args)]
pub struct GlobalArgs {
    #[arg(short, long, help = "tempdir to use for extractions/repacks. useful for debugging")]
    pub tmpdir: Option<PathBuf>,

    #[arg(long, help = "path to hactoolnet executable")]
    pub hactoolnet: Option<PathBuf>
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "print contents of an IMKV interpreted as a FS save index")]
    Print {
        #[arg(short, long, help = "a list of saveids to ignore")]
        filter_ids: Option<String>,
        #[arg(help = "path to a FS save index imkvdb.arc")]
        file: PathBuf,
    },
    #[command(about = "generate a FS save index save from a list of saves to index")]
    GenSave {
        #[arg(short, long, help = "the path in which to emit the resulding index save")]
        outdir: PathBuf,
        #[arg(help = "a list of saves to index")]
        saves: Vec<PathBuf>,
    },
    #[command(about = "index supplied saves and add it to an existing index save")]
    UpdateSave {
        #[arg(short, long, help = "the save to update")]
        save: PathBuf,
        #[arg(help = "a list of saves to index")]
        saves: Vec<PathBuf>,
    },
    #[command(long_about = "point at a SYSTEM mount where the FS save index save is missing an entry for the NCM content db and hope for the best")]
    FixSys {
        #[arg(help = "path to a SYSTEM mount (e.g. with hacdiskmount)")]
        sysmount: PathBuf,
    },
}
