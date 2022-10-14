use clap::{Parser,  ValueEnum};
use std::io::Write;
use std::cmp::min;

use saavn_rs::*;

use dialoguer::{
    FuzzySelect,
    theme::ColorfulTheme
};
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;

use futures_util::StreamExt;


const TEMPLATE: &str = " {msg} \n [{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec}";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Action {
    Search,
    Download,
    Play,
}

#[derive(Parser)]
#[command[author, version, about, long_about=None]]
struct Cli {
    #[arg(value_enum)]
    action: Action,
    #[arg(short, long)]
    name: Option<String>,
}

async fn download_song(final_url: String, song: String) {
    let response = reqwest::get(&final_url).await.unwrap();

    let total_size = response.content_length().ok_or(format!("Failed to get content length")).unwrap();

    //ProgressBar
    let pb = ProgressBar::new(total_size).with_message(format!("Downloading : {}", song));
    pb.set_style(ProgressStyle::default_bar().template(TEMPLATE).unwrap());

    let format = {
        if final_url.contains("mp4") {
            "mp4"
        } else {
            "mp3"
        }
    };

    let mut file = std::fs::File::create(format!("{}.{}",song,format)).unwrap();
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("error while downloading"))).unwrap();
        file.write_all(&chunk).unwrap();
        let new = min(downloaded+(chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }
}

async fn select_from_res(results: Results) {
    let song_strings: Vec<String> = results.results.iter().map(|item| format!("{} - {}", item.song, item.primary_artists)).collect();
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&song_strings)
        .default(0)
        .interact_on_opt(&Term::stderr()).unwrap();

    match selection  {
        Some(idx) => {
            let req_song = results.results.into_iter().nth(idx).unwrap();
            let final_url = convert_to_320(req_song.media_preview_url);
            println!("download url: {}", final_url);
            let name = req_song.song;
            download_or_play(name.to_string(), final_url).await;
        },
        None => println!("Nothing slected")
    }

}

async fn download_or_play(name: String, link: String) {
    let options = ["Play", "Download"];
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&options)
        .default(0)
        .interact_on_opt(&Term::stderr()).unwrap();

    match selection {
        Some(idx) => {
            if idx == 0 {
                play_link(link);
            } else {
                download_song(link, name).await;
            }
        },
        None => {
            println!("Nothing was selected");
        }
    }

}

fn play_link(link: String) {
    Command::new("mpv")
        .arg(link)
        .spawn()
        .expect("Mpv command failed");
}

#[tokio::main]
async fn main() -> Result<(), eyre::Report> {
    let cli = Cli::parse();
    let name:String = {
        if let Some(name) = cli.name.as_deref() {
            name.to_string()
        } else {
            panic!("Needs some name!")
        }
    };


    match cli.action {
        Action::Search => {
            let ress: Results = match get_all_res(name).await {
                Ok(v) => v,
                Err(e) => return Err(e)
            };
            select_from_res(ress).await;
        },
        Action::Download => {
            let (link,song) = get_download_link_name(name).await?;
            download_song(link, song).await;
        }, 
        Action::Play => {
            let link = get_download_link_name(name).await?.0;
            play_link(link);
        }
    };

    Ok(())
}
