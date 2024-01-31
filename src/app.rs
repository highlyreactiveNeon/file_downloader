use reqwest::blocking::Client;
use url::Url;

mod downloader;
// mod input;

pub fn run() {
    let client = Client::new();

    // let url = input::get_url_from_stdin();
    // let url = Url::parse("https://cdn.videvo.net/videvo_files/video/premium/video0042/large_watermarked/900-2_900-6334-PD2_preview.mp4").unwrap();
    let url = Url::parse("http://127.0.0.1:5500/src/test_input.txt").unwrap();

    let file = downloader::File::from_url(url, &client);
    file.download(3);
}