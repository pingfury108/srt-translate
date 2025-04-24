use clap::Parser;
use indicatif::ProgressBar;
use rig::completion::Prompt;
use rig::providers::deepseek;
use srtlib::{Subtitle, Subtitles};
use std::{fs, path::Path, process};

/// srt-translate subtitle translation tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = String::from("zh"),)]
    target_lang: String,

    #[arg(short, long)]
    src_file: String,

    #[arg(short, long)]
    to_file: Option<String>,

    #[arg(short, long)]
    deepseek_api_key: String,

    #[arg(long, default_value_t = String::from("deepseek-chat"))]
    deepseek_model: String,

    #[arg(long, default_value_t = false)]
    only_print: bool,

    #[arg(long, default_value_t = 3)]
    max_retries: u32,
}

fn parse_filename_ext(to_file: &str) -> (String, String) {
    let p = Path::new(to_file);
    let dir = match p.parent() {
        Some(s) => match s.to_str() {
            Some(s) => String::from(s),
            None => {
                log::warn!("Could not convert parent directory to string, using empty string");
                String::from("")
            }
        },
        None => String::from(""),
    };
    let name = match p.file_stem() {
        Some(s) => match s.to_str() {
            Some(s) => String::from(s),
            None => {
                log::warn!("Could not convert file stem to string, using empty string");
                String::from("")
            }
        },
        None => String::from(""),
    };
    let ext = match p.extension() {
        Some(s) => match s.to_str() {
            Some(s) => String::from(s),
            None => {
                log::warn!("Could not convert extension to string, using empty string");
                String::from("")
            }
        },
        None => String::from(""),
    };
    
    // 避免在dir为空时生成以/开头的路径
    let file_path = if dir.is_empty() {
        name
    } else {
        format!("{}/{}", dir, name)
    };
    
    return (file_path, ext);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    
    let subs = match Subtitles::parse_from_file(&args.src_file, None) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to read SRT file {}: {}", args.src_file, e);
            return;
        }
    };

    let (to_file, to_file_ext) = match args.to_file {
        Some(s) => parse_filename_ext(&s),
        None => parse_filename_ext(&args.src_file),
    };

    let temp_file_path = format!("{}.{}.temp", to_file, to_file_ext);
    let mut new_subs = if Path::new(&temp_file_path).exists() {
        match Subtitles::parse_from_file(&temp_file_path, None) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to read temporary SRT file {}: {}", temp_file_path, e);
                Subtitles::new()
            }
        }
    } else {
        Subtitles::new()
    };

    if args.only_print {
        println!("{}", subs);
        return;
    }

    // Create DeepSeek client and agent
    let deepseek_client = deepseek::Client::new(&args.deepseek_api_key);
    let model = deepseek_client.agent(&args.deepseek_model).build();

    let start_index = new_subs.len();
    let subs_vec = subs.to_vec();

    let bar = ProgressBar::new(subs_vec.len() as u64 - start_index as u64);
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

        // Retry mechanism for translation
        let mut retry_count = 0;
        let mut translation_successful = false;

        while retry_count <= args.max_retries && !translation_successful {
            // Translate using DeepSeek model
            let prompt = format!(
                "Translate this subtitle to {}:\n{}\n\nContext (not to be translated):\nPrevious: {}\nNext: {}\n\nImportant: Preserve all formatting, numbers, special characters, and non-text elements from the original subtitle exactly as they appear. Only translate the actual text content.\n\nOnly output the translation, nothing else.",
                args.target_lang, s.text, above_text, below_text
            );

            match model.prompt(prompt).await {
                Ok(translated_text) => {
                    log::debug!("{}", translated_text);
                    let translated_sub = Subtitle::new(s.num, s.start_time, s.end_time, translated_text);
                    new_subs.push(translated_sub.clone());
                    new_subs.sort();

                    if let Err(e) = new_subs.write_to_file(&temp_file_path, None) {
                        log::error!("Failed to write temporary SRT file {}: {}", temp_file_path, e);
                    } else {
                        log::debug!("Saved temporary SRT file {}", &temp_file_path);
                    }
                    translation_successful = true;
                }
                Err(e) => {
                    retry_count += 1;
                    log::warn!("Translation error (attempt {}/{}): {:?}", 
                              retry_count, args.max_retries + 1, e);
                    
                    if retry_count > args.max_retries {
                        log::error!("Maximum retry attempts ({}) exceeded. Exiting program.", 
                                   args.max_retries + 1);
                        process::exit(1);
                    }
                    
                    // Short delay before retrying
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
    }
    new_subs.sort();
    bar.finish();
    
    let final_file_path = format!("{}-translate.{}", to_file, to_file_ext);
    if let Err(e) = new_subs.write_to_file(&final_file_path, None) {
        log::error!("Failed to write translated SRT file {}: {}", final_file_path, e);
    } else {
        log::info!("Successfully created translated file: {}", final_file_path);
    }

    if let Err(e) = fs::remove_file(&temp_file_path) {
        log::warn!("Failed to remove temporary file {}: {}", temp_file_path, e);
    }
}
