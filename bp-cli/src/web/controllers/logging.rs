use std::io::SeekFrom;

use tide::http::mime;
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncSeekExt},
};

use crate::web::{constants::DEFAULT_LOG_FILE, state::State, utils::compress::gzip};

pub struct LoggingController;

impl LoggingController {
    pub async fn tail(req: tide::Request<State>) -> tide::Result {
        const TAIL_N: usize = 50;
        const BUFFER_SIZE: usize = 128;

        let mut file = fs::OpenOptions::new().read(true).open(DEFAULT_LOG_FILE).await.unwrap();

        let metadata = file.metadata().await?;
        let file_len = metadata.len() as usize;

        let mut buf = [0u8; BUFFER_SIZE];
        let mut line_count = 0usize;
        let mut iter_count = 0usize;
        let mut total_len = 0usize;
        let mut content_seg = Vec::with_capacity(TAIL_N);

        loop {
            if line_count >= TAIL_N as usize {
                break;
            }

            let seek = (BUFFER_SIZE * (iter_count + 1)) as i64;
            let mut seek_done = false;

            if file.seek(SeekFrom::End(-seek)).await.is_err() {
                seek_done = true;
                file.rewind().await?;
            }

            let n = file.read(&mut buf).await?;
            let end = if seek_done { file_len - total_len } else { n };
            let content = String::from_utf8(buf[0..end].to_vec())?;

            // count line breaks
            let breaks = content.chars().filter(|&ch| ch == '\n').count();

            content_seg.push(content);

            line_count += breaks;
            iter_count += 1;
            total_len += end;

            if seek_done || n < BUFFER_SIZE {
                break;
            }
        }

        content_seg.reverse();

        let full_content = content_seg.join("");

        let res = tide::Response::builder(200)
            .content_type(mime::PLAIN)
            .body(full_content)
            .build();

        gzip(&req, res)
    }
}
