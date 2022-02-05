use tokio::fs;

pub async fn read_file(path: &str) -> String {
    let buf = fs::read(path).await.unwrap();
    String::from_utf8(buf).unwrap()
}
