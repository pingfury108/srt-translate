use clap::Parser;
use dify_client::{request, Client, Config};
use indicatif::ProgressBar;
use srtlib::{Subtitle, Subtitles};
use std::{collections::HashMap, fs::{self, OpenOptions}, io::{self, BufRead, BufReader, Write}, path::Path, time::Duration};

/// srt-translate subtitle translation tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from("zh"),)]
    target_lang: String,

    #[arg(short, long)]
    src_file: String,

    #[arg(short, long)]
    to_file: String,

    #[arg(short, long)]
    dify_app_token: String,

    #[arg(long, default_value_t = String::from("https://api.dify.ai"))]
    dify_base_url: String,

    #[arg(long, default_value_t = false)]
    only_print: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let subs = Subtitles::parse_from_file(&args.src_file, None).expect("read srt file");

    let temp_file_path = format!("{}.temp", args.to_file);
    let mut new_subs = if Path::new(&temp_file_path).exists() {
        let file = fs::File::open(&temp_file_path).expect("open temp file");
        let reader = BufReader::new(file);
        let mut subs = Subtitles::new();
        for line in reader.lines() {
            let line = line.expect("read line");
            if let Ok(sub) = Subtitle::from_str(&line) {
                subs.push(sub);
            }
        }
        subs
    } else {
        Subtitles::new()
    };

    if args.only_print {
        println!("{}", subs);
        return;
    }

    let config = Config {
        base_url: args.dify_base_url,
        api_key: args.dify_app_token,
        timeout: Duration::from_secs(60),
    };
    let client = Client::new_with_config(config);

    let start_index = new_subs.len();
    let subs_vec = subs.to_vec();
    //let subs_vec = &subs_vec[0..50];

    let bar = ProgressBar::new(subs_vec.len() as u64);
    for (index, s) in subs_vec.iter().enumerate().skip(start_index) {
        bar.inc(1);
        new_subs.push(s.clone());

        let above_text: String = {
            if index == 0 {
                "".to_string()
            } else {
                subs_vec[index - 1].text.clone()
            }
        };

        let below_text: String = {
            if index == subs_vec.len() - 1 {
                "".to_string()
            } else {
                subs_vec[index + 1].text.clone()
            }
        };

        let data = request::CompletionMessagesRequest {
            user: "srt-translate".into(),
            response_mode: request::ResponseMode::Blocking,
            inputs: {
                let mut input = HashMap::new();
                input.insert("query".into(), s.text.clone());
                input.insert("lang".into(), args.target_lang.clone());
                input.insert("above".into(), above_text);
                input.insert("below".into(), below_text);

                input
            },
            ..Default::default()
        };

        let result = client.api().completion_messages(data).await;
        match result {
            Ok(r) => {
                log::debug!("{}", r.answer);
                let translated_sub = Subtitle::new(s.num, s.start_time, s.end_time, r.answer);
                new_subs.push(translated_sub.clone());

                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&temp_file_path)
                    .expect("open temp file for writing");
                writeln!(file, "{}", translated_sub).expect("write to temp file");
            }
            Err(e) => {
                log::error!("{}", e)
            }
        }
    }
    new_subs.sort();
    bar.finish();
    new_subs
        .write_to_file(args.to_file, None)
        .expect("write new srt file");

    fs::remove_file(temp_file_path).expect("remove temp file");
}
