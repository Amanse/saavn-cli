use clap::{Parser,  ValueEnum};

use saavn_rs::*;

use dialoguer::{
    FuzzySelect,
    theme::ColorfulTheme
};
use console::Term;
use std::process::Command;

use trauma::{download::Download, downloader::DownloaderBuilder};
use url::Url;
use std::path::PathBuf;

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
    name: Option<Vec<String>>,
}

async fn download_song(link_and_names: Vec<(String,String)>) {
    //ProgressBar
    // let pb = ProgressBar::new(total_size).with_message(format!("Downloading : {}", song));
    // pb.set_style(ProgressStyle::default_bar().template(TEMPLATE).unwrap());

    let mut downloads = vec![];
    for (final_url, song) in link_and_names {
    let format = {
        if final_url.contains("mp4") {
            "mp4"
        } else {
            "mp3"
        }
    };

    downloads.push(Download::new(&Url::parse(&final_url).unwrap(), &format!("{}.{}", song, format)))
    }
    let downloader_boi = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .build();
    downloader_boi.download(&downloads).await;
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
            let final_url = convert_to_320(req_song.media_preview_url).await.unwrap();
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
                play_link(vec![(link,name)]);
            } else {
                download_song(vec![(link, name)]).await;
            }
        },
        None => {
            println!("Nothing was selected");
        }
    }

}

fn play_link(links: Vec<(String,String)>) {
    let mut mpv = Command::new("mpv");
    for link in links {
        mpv.arg(link.0);
    }
    mpv.spawn().unwrap();
}

#[tokio::main]
async fn main() -> Result<(), eyre::Report> {
    let cli = Cli::parse();
    let names:Vec<String> = cli.name.unwrap();

    match cli.action {
        Action::Search => {
            let ress: Results = match get_all_res(names.into_iter().nth(0).unwrap()).await {
                Ok(v) => v,
                Err(e) => return Err(e)
            };
            select_from_res(ress).await;
        },
        Action::Download => {
            let link_and_names = get_download_link_name(names).await?;
            download_song(link_and_names).await;
        }, 
        Action::Play => {
            let link = get_download_link_name(names).await?;
            play_link(link);
        }
    };

    Ok(())
}
