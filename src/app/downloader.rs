use reqwest::{
    blocking::Client,
    header::{HeaderMap, CONTENT_LENGTH, RANGE},
};
use std::{fs, io, path::Path, thread};
use url::Url;

pub struct File<'c> {
    url: Url,
    file_name: String,
    content_length: u32,
    client: &'c Client,
}

impl<'c> File<'c> {
    pub fn from_url(url: Url, client: &'c Client) -> Self {
        let res = client
            .head(url.as_str())
            .send()
            .expect("Failed to get file metadata via HEAD request");

        let content_len = res
            .headers()
            .get(CONTENT_LENGTH)
            .expect("Content length header not found")
            .to_str()
            .expect("Failed to convert content length header value to string")
            .parse::<u32>()
            .expect("Failed to parse content length header value as u64");

        let file_name = Path::new(url.path())
            .file_name()
            .expect("Failed to get file name")
            .to_string_lossy()
            .to_string();

        Self {
            url,
            file_name,
            content_length: content_len,
            client,
        }
    }

    pub fn download(&self, num_chunks: u32) {
        let chunk_size = (self.content_length as f64 / num_chunks as f64).ceil() as u32;

        fs::create_dir_all("temp").expect("Failed to create temp directory");

        thread::scope(|s| {
            for i in 0..num_chunks - 1 {
                s.spawn(move || {
                    let low = i * chunk_size;
                    let high = low + chunk_size - 1;

                    let mut headers = HeaderMap::new();
                    headers.insert(RANGE, format!("bytes={}-{}", low, high).parse().unwrap());

                    let res = self
                        .client
                        .get(self.url.as_str())
                        .headers(headers)
                        .send()
                        .expect("Failed to get file chunk");

                    let temp_file_name = format!("{}.tmp", i);
                    let temp_file = fs::File::create(format!("temp/{}", temp_file_name))
                        .expect("Failed to create temp file");

                    let mut writer = io::BufWriter::new(temp_file);
                    let mut reader = io::BufReader::new(res);

                    io::copy(&mut reader, &mut writer)
                        .expect(format!("Failed to copy chunk {} to file", i).as_str());
                });
            }

            s.spawn(move || {
                let low = (num_chunks - 1) * chunk_size;
                let high = self.content_length - 1;

                let mut headers = HeaderMap::new();
                headers.insert(RANGE, format!("bytes={}-{}", low, high).parse().unwrap());

                let res = self
                    .client
                    .get(self.url.as_str())
                    .headers(headers)
                    .send()
                    .expect("Failed to get file chunk");

                let temp_file_name = format!("{}.tmp", num_chunks - 1);
                let temp_file = fs::File::create(format!("temp/{}", temp_file_name))
                    .expect("Failed to create temp file");

                let mut writer = io::BufWriter::new(temp_file);
                let mut reader = io::BufReader::new(res);

                io::copy(&mut reader, &mut writer)
                    .expect(format!("Failed to copy chunk {} to file", num_chunks - 1).as_str());
            });
        });

        let output_file = fs::File::create(self.file_name.as_str())
            .expect("Failed to create output file");

        let mut writer = io::BufWriter::new(output_file);

        for i in 0..num_chunks {
            let temp_file_name = format!("{}.tmp", i);
            let temp_file = fs::File::open(format!("temp/{}", temp_file_name))
                .expect("Failed to open temp file");

            let mut reader = io::BufReader::new(temp_file);

            io::copy(&mut reader, &mut writer)
                .expect(format!("Failed to copy chunk {} to file", i).as_str());
        }

        fs::remove_dir_all("temp").expect("Failed to remove temp directory");
    }
}