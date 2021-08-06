use env_logger::{Target, WriteStyle};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::SystemTime;

pub async fn setup() {
    let file_path = get_file_path();

    env_logger::builder()
        .default_format()
        .write_style(WriteStyle::Always)
        // file never have content due to this bug of Target:
        // https://github.com/env-logger-rs/env_logger/issues/208
        .target(Target::Pipe(Box::new(FileTarget::new(file_path))))
        .init();
}

// return ~/.bp/logs/bp-{date}.log
fn get_file_path() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(".bp");
    dir.push("logs");

    // get date string, format is YYYY-MM-DD
    let date = humantime::format_rfc3339_millis(SystemTime::now());
    let mut date_str = date.to_string();
    let _ = date_str.split_off(date_str.find('T').unwrap());

    dir.push(format!("bp-{}.log", date_str));
    dir
}

struct FileTarget {
    file_writer: Option<BufWriter<File>>,
}

impl FileTarget {
    pub fn new(name: PathBuf) -> FileTarget {
        let file_writer = if let Ok(file) = Self::open(name) {
            Some(BufWriter::new(file))
        } else {
            None
        };
        FileTarget { file_writer }
    }

    fn open(file: PathBuf) -> Result<File, std::io::Error> {
        create_dir_all(file.parent().unwrap())?;
        OpenOptions::new().write(true).create(true).append(true).open(file)
    }
}

impl Drop for FileTarget {
    fn drop(&mut self) {
        self.flush().unwrap();
    }
}

impl Write for FileTarget {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Some(writer) = &mut self.file_writer {
            writer.write_all(buf)?;
        }
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(writer) = &mut self.file_writer {
            writer.flush()
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_get_file_name() {
    let path = get_file_path();
    let path = path.to_str().unwrap();
    let parts: Vec<&str> = path.split(std::path::MAIN_SEPARATOR).collect();

    assert!(parts.contains(&".bp"));
    assert!(parts.contains(&"logs"));
}
