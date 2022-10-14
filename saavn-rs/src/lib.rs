use serde::Deserialize;

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

//@TODO
//Make error is no mp3 or mp4 is found
//Make all returns Result or eyre result for handling errors


//NEEDS
//1. Function that returns link from first res for given name
//2. Function that returns vector of songs
//3. Function to convert 96kpbs link to 320kbps link

//Public fn that will interact and send the downnload link to the main app
pub async fn get_download_link_name(name: String) -> (String, String) {
    let res = first_res(name).await;
    //@TODO
    //Check if mp4 or mp3 is the correct link and send that
    (convert_to_320(res.media_preview_url), res.song)
}

//Public fn that will return vector of search results to the main app
pub async fn get_all_res(name: String) -> Results {
    let body: String = reqwest::get(&format!("{}{}", SEARCH_URL, name))
        .await.unwrap()
        .text()
        .await.unwrap();

    serde_json::from_str(&body).unwrap()
}

pub fn convert_to_320(link: String) -> String {
    link.replace("preview", "h").replace("_96_p.mp4", "_320.mp4")
}

//Function that returns the first result for playing or downloading
async fn first_res(name: String) -> Song {
    get_all_res(name).await.results.into_iter().nth(0).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::get_download_link_name;

    #[tokio::test]
    async fn check_download() {
        let link = get_download_link_name("pasoori".to_string()).await.0;
        let dl_link = String::from("http://h.saavncdn.com/663/4aef67fd9511a82b1f49835101c145a7_320.mp4");
        assert_eq!(dl_link, link);
    }
}
