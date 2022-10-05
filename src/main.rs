use std::fmt::format;

use clap::{Parser, Subcommand, ValueEnum};
use serde_json::{Result, Value};
use serde::Deserialize;

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
   id: String,
   song: String,
   image: String,
   media_preview_url: String,
}

#[derive(Deserialize)]
struct Results {
    results: Vec<Song>,
}


fn get_download_link(name: String) -> String {
    let body: String = ureq::get(&format!("{}{}", SEARCH_URL, name))
        .set("Example-Header", "header value")
        .call().unwrap()
        .into_string().unwrap();

    let search_res: Value = serde_json::from_str(&body).unwrap();

    println!("{}", search_res["results"][0]["media_preview_url"]);

    todo!()

}

fn main() {
    let cli = Cli::parse();
    let name:String = {
        if let Some(name) = cli.name.as_deref() {
            name.to_string()
        } else {
            panic!("Needs some name!")
        }
    };

    let action: Action = cli.action;

    get_download_link(name);
}
