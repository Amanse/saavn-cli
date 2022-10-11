use clap::{Parser,  ValueEnum};
use std::io::{Write};
use std::cmp::min;
use serde::Deserialize;

use dialoguer::{
    FuzzySelect,
    theme::ColorfulTheme
};
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;
use std::process::Command;



const SEARCH_URL: &str = "https://www.jiosaavn.com/api.php?_format=json&n=5&p=1&_marker=0&ctx=android&__call=search.getResults&q=";
const TEMPLATE: &str = "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {bytes_per_sec}";

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

#[derive(Deserialize)]
struct Song { 
   song: String,
   media_preview_url: String,
   primary_artists: String,
}

#[derive(Deserialize)]
struct Results {
    results: Vec<Song>,
}


async fn search_res(name: String) -> Results {
    let body: String = reqwest::get(&format!("{}{}", SEARCH_URL, name))
        .await.unwrap()
        .text()
        .await.unwrap();

    serde_json::from_str(&body).unwrap()
}

async fn get_download_link(temp_link: String) -> String {
    let temp_url = temp_link.replace("preview", "h");
    let final_url = temp_url.replace("_96_p.mp4", "_320.mp4");
    fix_link(final_url).await
}

async fn download_song(final_url: String, song: String) {
    let response = reqwest::get(&final_url).await.unwrap();

    let total_size = response.content_length().ok_or(format!("Failed to get content length")).unwrap();

    //ProgressBar
    let pb = ProgressBar::new(total_size);
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
            let temp_url = &results.results[idx].media_preview_url;
            let final_url = get_download_link(temp_url.to_string()).await;
            println!("download url: {}", final_url);
            let name = &results.results[idx].song;
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

async fn fix_link(link: String) -> String {
    let resp = reqwest::get(&link).await.unwrap();
    if resp.status() == 404 {
        link.replace("mp4", "mp3")
    } else {
        link
    }
}

#[tokio::main]
async fn main() {
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
            let results = search_res(name).await;
            select_from_res(results).await;
        },
        Action::Download => {
            let results = search_res(name).await;
            let name = &results.results[0].song;
            let temp_link = &results.results[0].media_preview_url;
            let link =  get_download_link(temp_link.to_string()).await;
            download_song(link, name.to_string()).await;
        }, 
        Action::Play => {
            let results = search_res(name).await;
            let temp_link = &results.results[0].media_preview_url;
            let link =  get_download_link(temp_link.to_string()).await;
            play_link(link);
        }
    }
}
