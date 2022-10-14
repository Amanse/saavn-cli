use serde::Deserialize;
use eyre::{Result, eyre};

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

//NEEDS
//1. Function that returns link from first res for given name
//2. Function that returns vector of songs
//3. Function to convert 96kpbs link to 320kbps link

//Public fn that will interact and send the downnload link to the main app
pub async fn get_download_link_name(name: String) -> Result<(String, String)> {
    let res = first_res(name).await?;
    let mut temp_url = convert_to_320(res.media_preview_url);
    let resp = reqwest::get(&temp_url).await?;
    if resp.status() == 404 {
        temp_url = temp_url.replace("mp4", "mp3");
        let resp = reqwest::get(&temp_url).await?;
        if resp.status() == 404 {
            return Err(eyre!("Song not available!"))
        } else {
            return Ok((temp_url, res.song))
        }
    } else {
        return Ok((temp_url, res.song))
    }
}

//Public fn that will return vector of search results to the main app
pub async fn get_all_res(name: String) -> Result<Results> {
    let body: String = reqwest::get(&format!("{}{}", SEARCH_URL, name))
        .await?
        .text()
        .await?;

    let res = serde_json::from_str::<Results>(&body)?;
    if res.results.len() == 0 {
        return Err(eyre!("No songs found"));
    }
    Ok(res)
}

pub fn convert_to_320(link: String) -> String {
    link.replace("preview", "h").replace("_96_p.mp4", "_320.mp4")
}

//Function that returns the first result for playing or downloading
async fn first_res(name: String) -> Result<Song> {
    get_all_res(name).await?.results.into_iter().nth(0).ok_or_else(|| eyre!("No songs in the result"))
}

#[cfg(test)]
mod tests {
    use crate::get_download_link_name;

    #[tokio::test]
    async fn check_download() {
        let link = get_download_link_name("told you so hanita".to_string()).await.unwrap().0;
        let dl_link = String::from("http://h.saavncdn.com/062/bdf7ab0f70309de97e0c6a169e7bc520_320.mp3");
        assert_eq!(dl_link, link);
    }

    #[tokio::test]
    async fn check_download_mp4() {
        let link = get_download_link_name(String::from("Pasoori")).await.unwrap().0;
        let dl_link = String::from("http://h.saavncdn.com/663/4aef67fd9511a82b1f49835101c145a7_320.mp4");
        assert_eq!(link, dl_link);
    }
}
