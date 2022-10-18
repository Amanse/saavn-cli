use serde::Deserialize;
use eyre::Result;
use thiserror::Error;

const SEARCH_URL: &str = "https://www.jiosaavn.com/api.php?_format=json&n=5&p=1&_marker=0&ctx=android&__call=search.getResults&q=";

#[derive(Deserialize)]
pub struct Song { 
   pub song: String,
   pub media_preview_url: String,
   pub primary_artists: String,
}

#[derive(Deserialize)]
pub struct Results {
   pub results: Vec<Song>,
}

#[derive(Error, Debug)]
pub enum SaavnRsErros {
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Number of songs must be between 1 and 15")]
    InvalidNumberOfSongs,
    #[error("Song is not available on JioSaavn, skipping")]
    SongNotFound,
    #[error("Request failure from api {0:?}")]
    RequestFailure(#[from] reqwest::Error),
    #[error("No songs in the result")] 
    NoSongInResult,
    #[error("Skipping")]
    SkipSong,
}

//NEEDS
//1. Function that returns link from first res for given name
//2. Function that returns vector of songs
//3. Function to convert 96kpbs link to 320kbps link

//Public fn that will interact and send the downnload link to the main app
pub async fn get_download_link_name(names: Vec<String>) -> Result<Vec<(String, String)>, SaavnRsErros> {
    if names.len() >= 15 || names.len() < 1 {
        return Err(SaavnRsErros::InvalidNumberOfSongs);
    }  else {
        let mut links_and_names: Vec<(String,String)> = vec![];
        for name in names {
        let res = first_res(name.to_string()).await?;
        let final_url = match convert_to_320(res.media_preview_url).await {
            Ok(v) => v,
            Err(e) => {
                match e {
                    SaavnRsErros::SongNotFound => {println!("Skipping {}", name); continue;},
                    _ => return Err(e)
                }
            },
        };
        links_and_names.push((final_url, res.song))
        }
       Ok(links_and_names)
    }

}

//Public fn that will return vector of search results to the main app
pub async fn get_all_res(name: String) -> Result<Results, SaavnRsErros> {
    let body: String = reqwest::get(&format!("{}{}", SEARCH_URL, name))
        .await?
        .text()
        .await?;

    let res = serde_json::from_str::<Results>(&body)?;
    if res.results.len() == 0 {
        return Err(SaavnRsErros::NoSongInResult);
    }
    Ok(res)
}

pub async fn convert_to_320(link: String) -> Result<String, SaavnRsErros> {
    Ok(handle_mp3(link.replace("preview", "h").replace("_96_p.mp4", "_320.mp4")).await?)
}

//Function that returns the first result for playing or downloading
async fn first_res(name: String) -> Result<Song, SaavnRsErros> {
    match get_all_res(name).await?.results.into_iter().nth(0) {
        Some(v) => Ok(v),
        None => Err(SaavnRsErros::NoSongInResult)
    }
    
}

async fn handle_mp3(temp_url: String) -> Result<String, SaavnRsErros> {
    let resp = reqwest::get(&temp_url).await?;
    if resp.status() == 404 {
        let final_url = temp_url.replace("mp4", "mp3");
        let resp = reqwest::get(&final_url).await?;
        if resp.status() == 404 {
            return Err(SaavnRsErros::SongNotFound)
        } else {
            return Ok(final_url)
        }
    } else {
        return Ok(temp_url)
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::get_download_link_name;

    #[tokio::test]
    async fn check_download() {
        let link = get_download_link_name(vec![String::from("told you so hanita"), String::from("Pasoori")]).await.unwrap();
        let told_you_dl_link = String::from("http://h.saavncdn.com/062/bdf7ab0f70309de97e0c6a169e7bc520_320.mp3");
        let told_you_name = String::from("Told You So");
        let pasoori_url = String::from("http://h.saavncdn.com/663/4aef67fd9511a82b1f49835101c145a7_320.mp4");
        let pasoori_name = String::from("Pasoori");
        let res = vec![(told_you_dl_link, told_you_name), (pasoori_url, pasoori_name)];
        assert_eq!(res, link);
    }
}
