use clap::Parser;
use ctrlc;
use sqlite;
use sqlite::{Connection, State};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs, thread, time};
use walkdir::WalkDir;
// use clap::Arg;
use chrono::Local;
use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::sha1::Sha1;

#[derive(Parser, Debug)]
#[clap(author="2022@Riko", version, about="Small tool to watch disk change.", long_about = None)]
struct Args {
    /// Set the watch path
    #[clap(short, long, default_value = "./", default_missing_value = "./")]
    path: String,

    /// Display the database all values
    #[clap(short, long, default_value = "no", default_missing_value = "yes")]
    dbshow: String,
}

struct Database {
    connection: Connection,
}

impl Database {
    fn connect() -> Self {
        let connection = sqlite::open("diskwatch.db").unwrap();
        let sql_str = "CREATE TABLE IF NOT EXISTS files (path TEXT UNIQUE, md5 TEXT, sha1 TEXT, last_opt TEXT);";
        match connection.execute(sql_str) {
            _ => (),
        }
        Database {
            connection: connection,
        }
    }
    fn insert(&self, path: &String, md5: &String, sha1: &String, last_opt: &String) {
        // println!("{}", file_path);
        // println!("sha1 = {}", sha1);
        // println!("md5 = {}", md5);
        let sql_str = format!(
            "INSERT INTO files (path, md5, sha1, last_opt) VALUES ('{}', '{}', '{}', '{}');",
            path, md5, sha1, last_opt
        );
        match self.connection.execute(sql_str) {
            _ => (),
        }
    }
    fn update(&self, path: &String, md5: &String, sha1: &String, last_opt: &String) {
        // println!("sha1 = {}", sha1_hex);
        // println!("md5 = {}", md5_hex);
        let sql_str = format!(
            "UPDATE files SET md5='{}', sha1='{}', last_opt='{}' WHERE path='{}'",
            md5, sha1, last_opt, path
        );
        match self.connection.execute(sql_str) {
            _ => (),
        }
    }
    fn select(&self, path: &String) -> Vec<Vec<String>> {
        // let sql_str = format!("SELECT * FROM files;");
        let sql_str = format!("SELECT * FROM files WHERE path='{}';", path);
        let mut statement = self.connection.prepare(sql_str).unwrap();
        let mut result_v: Vec<Vec<String>> = Vec::new();

        while let State::Row = statement.next().unwrap() {
            let path = statement.read::<String>(0).unwrap();
            let md5 = statement.read::<String>(1).unwrap();
            let sha1 = statement.read::<String>(2).unwrap();
            let last_opt = statement.read::<String>(3).unwrap();
            /*
            println!("path = {}", statement.read::<String>(0).unwrap());
            println!("md5 = {}", statement.read::<String>(1).unwrap());
            println!("sha1 = {}", statement.read::<String>(2).unwrap());
            */
            let tmp_v: Vec<String> = vec![path, md5, sha1, last_opt];
            result_v.push(tmp_v);
        }
        result_v
    }
    fn select_all(&self) -> Vec<Vec<String>> {
        // let sql_str = format!("SELECT * FROM files;");
        let sql_str = format!("SELECT * FROM files;");
        let mut statement = self.connection.prepare(sql_str).unwrap();
        let mut result_v: Vec<Vec<String>> = Vec::new();

        while let State::Row = statement.next().unwrap() {
            let path = statement.read::<String>(0).unwrap();
            let md5 = statement.read::<String>(1).unwrap();
            let sha1 = statement.read::<String>(2).unwrap();
            let last_opt = statement.read::<String>(3).unwrap();
            /*
            println!("path = {}", statement.read::<String>(0).unwrap());
            println!("md5 = {}", statement.read::<String>(1).unwrap());
            println!("sha1 = {}", statement.read::<String>(2).unwrap());
            */
            let tmp_v: Vec<String> = vec![path, md5, sha1, last_opt];
            result_v.push(tmp_v);
        }
        result_v
    }
    fn show_all(&self) {
        let sql_str = format!("SELECT * FROM files;");
        let mut statement = self.connection.prepare(sql_str).unwrap();
        while let State::Row = statement.next().unwrap() {
            println!("path = {}", statement.read::<String>(0).unwrap());
            println!("md5 = {}", statement.read::<String>(1).unwrap());
            println!("sha1 = {}", statement.read::<String>(2).unwrap());
            println!("last_opt = {}", statement.read::<String>(3).unwrap());
        }
    }
}

fn md5_cal(data: &Vec<u8>) -> Option<String> {
    let mut md5_hasher = Md5::new();
    md5_hasher.input(data);
    Some(md5_hasher.result_str())
}

fn sha1_cal(data: &Vec<u8>) -> Option<String> {
    let mut sha1_hasher = Sha1::new();
    sha1_hasher.input(data);
    Some(sha1_hasher.result_str())
}

fn log_to_file(contents: &String) {
    let date = Local::now();
    let data_str = date.format("%Y-%m-%d %H:%M:%S");
    let target_file = "diskwatch.log";
    let log_str = format!("{} - {}", data_str, contents);
    if !Path::new(target_file).exists() {
        let mut file = fs::File::create(target_file).expect("Can not create the log file");
        writeln!(file, "{}", &log_str).expect("Can not write to log file");
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(target_file)
            .unwrap();
        if let Err(e) = writeln!(file, "{}", log_str) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
    println!("{}", log_str);
}

fn all_files(target_dir: &String) -> Vec<String> {
    let mut result_v: Vec<String> = Vec::new();
    if target_dir.len() > 0 {
        for entry in WalkDir::new(target_dir) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                // println!("{}", path.display());
                result_v.push(path.to_string_lossy().to_string())
            }
        }
    }
    result_v
}

fn forward(db: &Database, watchpath: &String) {
    /* Travel all files to find file change or file add
     */
    let all_files_v = all_files(watchpath);
    // println!("{:?}", all_files_v);
    for f in &all_files_v {
        let data = fs::read(f).expect("Something went wrong reading the file");
        let md5 = md5_cal(&data).unwrap();
        let sha1 = sha1_cal(&data).unwrap();
        let query_result_v = db.select(&f);
        if query_result_v.len() > 0 {
            for q in query_result_v {
                let md5_old = &q[1];
                let sha1_old = &q[2];
                if (*md5_old != md5) && (*sha1_old != sha1) {
                    let log_str = format!("Some file changed: {}", f);
                    log_to_file(&log_str);
                    // println!("update");
                    let last_opt = String::from("changed");
                    db.update(&f, &md5, &sha1, &last_opt);
                }
            }
        } else {
            // println!("insert");
            let log_str = format!("Some file added: {}", f);
            log_to_file(&log_str);
            let last_opt = String::from("Added");
            db.insert(&f, &md5, &sha1, &last_opt);
        }
    }
}

fn reverse(db: &Database, watchpath: &String) {
    /* Travel all database stored files to find file delete
     */
    let db_files_v = db.select_all();
    let all_files_v = all_files(watchpath);
    for df in db_files_v {
        let last_opt = &df[3];
        let path = &df[0];
        if !all_files_v.contains(path) && last_opt != "deleted" {
            let log_str = format!("Some file deleted: {}", path);
            let null = String::from("null");
            let last_opt = String::from("deleted");
            db.update(path, &null, &null, &last_opt);
            log_to_file(&log_str);
        }
    }
}

fn main() {
    let args = Args::parse();
    let db = Database::connect();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    if &args.dbshow != "no" {
        db.show_all();
    } else {
        println!("diskwach runing...");
        while running.load(Ordering::SeqCst) {
            forward(&db, &args.path);
            reverse(&db, &args.path);
            // sleep 1s
            let ten_millis = time::Duration::from_secs(1);
            thread::sleep(ten_millis);
        }
        println!("Exiting now...");
    }
}
