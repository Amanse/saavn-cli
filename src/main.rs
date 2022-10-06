use clap::{Parser,  ValueEnum};
use std::io::Cursor;
use serde::Deserialize;

use dialoguer::{
    FuzzySelect,
    theme::ColorfulTheme
};
use console::Term;


const SEARCH_URL: &str = "https://www.jiosaavn.com/api.php?_format=json&n=5&p=1&_marker=0&ctx=android&__call=search.getResults&q=";

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Action {
    Search,
    Download,
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
//   id: String,
   song: String,
//   image: String,
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

    final_url
}

async fn download_song(final_url: String, song: String) {
    let response = reqwest::get(final_url).await.unwrap();
    let mut file = std::fs::File::create(format!("{}.mp4",song)).unwrap();
    let mut content =  Cursor::new(response.bytes().await.unwrap());
    std::io::copy(&mut content, &mut file).unwrap();
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
            let song_strings: Vec<String> = results.results.iter().map(|item| format!("{} - {}", item.song, item.primary_artists)).collect();
            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .items(&song_strings)
                .default(0)
                .interact_on_opt(&Term::stderr()).unwrap();

            match selection  {
                Some(idx) => println!("Selected song is {}", song_strings[idx]),
                None => println!("Nothing slected")
            }
        },
        Action::Download => {
            let results = search_res(name).await;
            let name = &results.results[0].song;
            let temp_link = &results.results[0].media_preview_url;
            let link =  get_download_link(temp_link.to_string()).await;
            download_song(link, name.to_string()).await;
        }
    }
}
