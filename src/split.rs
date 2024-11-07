use clap::Parser;
use srtlib::{Subtitle, Subtitles};

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

    for (index, sub) in subs.to_vec().iter().enumerate() {
        println!("{}", sub.text);
        if index % 2 == 0 {
            subs_lang1.push(Subtitle::new(
                index,
                sub.start_time,
                sub.end_time,
                sub.text.clone(),
            ));
        } else {
            subs_lang2.push(Subtitle::new(
                index,
                sub.start_time,
                sub.end_time,
                sub.text.clone(),
            ));
        }
    }

    subs_lang1
        .write_to_file(format!("{}_lang1.srt", args.to_file), None)
        .expect("write lang1 srt file");
    subs_lang2
        .write_to_file(format!("{}_lang2.srt", args.to_file), None)
        .expect("write lang2 srt file");
}
