use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time;

use chrono::Local;
use clap::Parser;
use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use sqlite;
use sqlite::Connection;
use sqlite::State;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[clap(author="RikoNaka", version, about="Small tool to watch disk change.", long_about = None)]
struct Args {
    /// Set the watch path
    #[clap(short, long, default_value = "test", default_missing_value = "test")]
    watchpath: String,

    /// Display the database all values
    #[clap(short, long, action)]
    dbshow: bool,
}

struct Database {
    connection: Connection,
}

impl Database {
    fn connect() -> Result<Database, sqlite::Error> {
        let connection = sqlite::open("diskwatch.db").unwrap();
        let sql_str = "CREATE TABLE IF NOT EXISTS files (path TEXT UNIQUE, md5 TEXT, sha1 TEXT, last_opt TEXT);";
        match connection.execute(sql_str) {
            Ok(_) => Ok(Database { connection }),
            Err(e) => Err(e),
        }
    }
    fn insert(
        &self,
        path: &String,
        md5: &String,
        sha1: &String,
        last_opt: &String,
    ) -> Result<(), sqlite::Error> {
        let sql_str = format!(
            "INSERT INTO files (path, md5, sha1, last_opt) VALUES ('{}', '{}', '{}', '{}');",
            path, md5, sha1, last_opt
        );
        self.connection.execute(sql_str)
    }
    fn update(
        &self,
        path: &String,
        md5: &String,
        sha1: &String,
        last_opt: &String,
    ) -> Result<(), sqlite::Error> {
        let sql_str = format!(
            "UPDATE files SET md5='{}', sha1='{}', last_opt='{}' WHERE path='{}'",
            md5, sha1, last_opt, path
        );
        self.connection.execute(sql_str)
    }
    fn select(&self, path: &String) -> Result<Vec<Vec<String>>, sqlite::Error> {
        let sql_str = format!("SELECT * FROM files WHERE path='{}';", path);
        let mut statement = self.connection.prepare(sql_str)?;
        let mut result_v: Vec<Vec<String>> = Vec::new();

        while let State::Row = statement.next()? {
            let path = statement.read::<String, _>(0)?;
            let md5 = statement.read::<String, _>(1)?;
            let sha1 = statement.read::<String, _>(2)?;
            let last_opt = statement.read::<String, _>(3)?;
            let tmp_v: Vec<String> = vec![path, md5, sha1, last_opt];
            result_v.push(tmp_v);
        }
        Ok(result_v)
    }
    fn select_all(&self) -> Result<Vec<Vec<String>>, sqlite::Error> {
        let sql_str = format!("SELECT * FROM files;");
        let mut statement = self.connection.prepare(sql_str)?;
        let mut result_v: Vec<Vec<String>> = Vec::new();

        while let State::Row = statement.next()? {
            let path = statement.read::<String, _>(0)?;
            let md5 = statement.read::<String, _>(1)?;
            let sha1 = statement.read::<String, _>(2)?;
            let last_opt = statement.read::<String, _>(3)?;
            let tmp_v: Vec<String> = vec![path, md5, sha1, last_opt];
            result_v.push(tmp_v);
        }
        Ok(result_v)
    }
    fn show_all(&self) -> Result<(), sqlite::Error> {
        let sql_str = format!("SELECT * FROM files;");
        let mut statement = self.connection.prepare(sql_str)?;
        while let State::Row = statement.next()? {
            println!("path = {}", statement.read::<String, _>(0)?);
            println!("md5 = {}", statement.read::<String, _>(1)?);
            println!("sha1 = {}", statement.read::<String, _>(2)?);
            println!("last_opt = {}", statement.read::<String, _>(3)?);
        }
        Ok(())
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
        let mut file = fs::File::create(target_file).expect("can not create the log file");
        writeln!(file, "{}", &log_str).expect("can not write to log file");
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(target_file)
            .unwrap();
        if let Err(e) = writeln!(file, "{}", log_str) {
            eprintln!("couldn't write to file: {}", e);
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

/// Travel all files to find file change or file add
fn travel_files(db: &Database, watchpath: &String) -> Result<(), sqlite::Error> {
    let all_files_v = all_files(watchpath);
    // println!("{:?}", all_files_v);
    for f in &all_files_v {
        let data = fs::read(f).expect("something went wrong reading the file");
        let md5 = md5_cal(&data).unwrap();
        let sha1 = sha1_cal(&data).unwrap();
        let query_result_v = db.select(&f)?;
        if query_result_v.len() > 0 {
            for q in query_result_v {
                let md5_old = &q[1];
                let sha1_old = &q[2];
                if (*md5_old != md5) && (*sha1_old != sha1) {
                    let log_str = format!("File changed: {}", f);
                    log_to_file(&log_str);
                    // println!("update");
                    let last_opt = String::from("changed");
                    let _ = db.update(&f, &md5, &sha1, &last_opt)?;
                }
            }
        } else {
            let log_str = format!("File added: {}", f);
            log_to_file(&log_str);
            let last_opt = String::from("added");
            let _ = db.insert(&f, &md5, &sha1, &last_opt)?;
        }
    }
    Ok(())
}

/// Travel all database stored files to find file delete
fn travel_dbs(db: &Database, watchpath: &String) -> Result<(), sqlite::Error> {
    let db_files_v = db.select_all()?;
    let all_files_v = all_files(watchpath);
    for df in db_files_v {
        let last_opt = &df[3];
        let path = &df[0];
        if !all_files_v.contains(path) && last_opt != "deleted" {
            let log_str = format!("File deleted: {}", path);
            let null = String::from("null");
            let last_opt = String::from("deleted");
            let _ = db.update(path, &null, &null, &last_opt)?;
            log_to_file(&log_str);
        }
    }
    Ok(())
}

fn main() -> Result<(), sqlite::Error> {
    let args = Args::parse();
    let db = Database::connect()?;

    if args.dbshow {
        let _ = db.show_all()?;
    } else {
        println!("diskwach runing...");
        loop {
            let _ = travel_files(&db, &args.watchpath)?;
            let _ = travel_dbs(&db, &args.watchpath)?;
            // sleep
            let ten_millis = time::Duration::from_secs_f32(0.5);
            thread::sleep(ten_millis);
        }
        // println!("exiting now...");
    }
    Ok(())
}
