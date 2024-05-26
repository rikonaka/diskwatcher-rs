use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::Local;
use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use walkdir::WalkDir;

use super::LOG_FILE;

pub fn md5_cal(data: &[u8]) -> String {
    let mut md5_hasher = Md5::new();
    md5_hasher.input(data);
    md5_hasher.result_str()
}

pub fn sha1_cal(data: &[u8]) -> String {
    let mut sha1_hasher = Sha1::new();
    sha1_hasher.input(data);
    sha1_hasher.result_str()
}

pub fn log_to_file(text: &str) {
    let date = Local::now();
    let data_str = date.format("%Y-%m-%d %H:%M:%S");
    let log_str = format!("{} - {}", data_str, text);
    if !Path::new(LOG_FILE).exists() {
        let mut file = fs::File::create(LOG_FILE).expect("can not create the log file");
        writeln!(file, "{}", &log_str).expect("can not write to log file");
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(LOG_FILE)
            .unwrap();
        if let Err(e) = writeln!(file, "{}", log_str) {
            eprintln!("couldn't write to file: {}", e);
        }
    }
    println!("{}", log_str);
}

pub fn disk_files_and_folders(watchpath: &str) -> (Vec<String>, Vec<String>) {
    let mut files = Vec::new();
    let mut folders = Vec::new();
    if watchpath.len() > 0 {
        for entry in WalkDir::new(watchpath) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                // println!("{}", path.display());
                files.push(path.display().to_string())
            } else if path.is_dir() {
                folders.push(path.display().to_string())
            }
        }
    }
    (files, folders)
}
