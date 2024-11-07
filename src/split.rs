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

    let start_index = new_subs.len();
    let subs_vec = subs.to_vec();
    //let subs_vec = &subs_vec[new_subs.len() - 1..subs.len()];

    let bar = ProgressBar::new(subs_vec.len() as u64);
    for (index, s) in subs_vec.iter().enumerate().skip(start_index) {
        bar.inc(1);
    }
    new_subs.sort();
    bar.finish();
    new_subs
        .write_to_file(args.to_file, None)
        .expect("write new srt file");
}
