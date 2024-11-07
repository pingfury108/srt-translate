use clap::Parser;
use dify_client::{request, Client, Config};
use indicatif::ProgressBar;
use srtlib::{Subtitle, Subtitles};
use std::{collections::HashMap, fs, path::Path, time::Duration};

/// split subtitle
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    src_file: String,

    #[arg(short, long)]
    to_file: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let subs = Subtitles::parse_from_file(&args.src_file, None).expect("read srt file");

    let mut subs_lang1 = Subtitles::new();
    let mut subs_lang2 = Subtitles::new();

    for sub in subs.iter() {
        let parts: Vec<&str> = sub.text.split('\n').collect();
        if parts.len() == 2 {
            subs_lang1.push(Subtitle::new(sub.index, sub.start, sub.end, parts[0].to_string()));
            subs_lang2.push(Subtitle::new(sub.index, sub.start, sub.end, parts[1].to_string()));
        }
    }

    subs_lang1.write_to_file(format!("{}_lang1.srt", args.to_file), None).expect("write lang1 srt file");
    subs_lang2.write_to_file(format!("{}_lang2.srt", args.to_file), None).expect("write lang2 srt file");
}
