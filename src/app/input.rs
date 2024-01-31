use url::Url;

pub fn get_url_from_stdin() -> Url {
    let mut url = String::new();
    println!("Enter URL: ");
    std::io::stdin()
        .read_line(&mut url)
        .expect("Failed to read url from stdin");

    let parsed_url = Url::parse(&url).expect("Failed to parse url");

    parsed_url
}
