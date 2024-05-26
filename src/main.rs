use chrono::Datelike;
use chrono::Timelike;
use clap::Parser;
use db::CLASS;
use db::OPT;
use rusqlite;
use std::fs;
use std::thread;
use std::time;

mod db;
mod utils;

use db::WatchDB;
use utils::disk_files_and_folders;
use utils::log_to_file;
use utils::md5_cal;
use utils::sha1_cal;

const DB_FILE: &str = "diskwatcher.db";
const LOG_FILE: &str = "diskwatcher.log";

#[derive(Parser, Debug)]
#[clap(author="RikoNaka", version, about="Small tool to watch disk files change.", long_about = None)]
struct Args {
    /// Set the watch path
    #[clap(short, long, default_value = "test", default_missing_value = "test")]
    watch: String,
    /// Start and flush old data
    #[clap(short, long, action)]
    flush: bool,
    /// Display the database
    #[clap(short, long, action)]
    printdb: bool,
}

/// Travel all files to find file change or file add
fn travel_disk(db: &WatchDB, watchpath: &str) -> Result<(), rusqlite::Error> {
    let (disk_files, disk_floders) = disk_files_and_folders(&watchpath);
    for f in disk_files {
        let r = db.select(&f)?;
        let data = match fs::read(&f) {
            Ok(d) => d,
            Err(e) => {
                let log_str = format!("Read file failed '{}': {}", &f, e);
                log_to_file(&log_str);
                vec![]
            }
        };
        let md5 = md5_cal(&data);
        let sha1 = sha1_cal(&data);

        if r.len() == 0 {
            // database no such file
            db.insert(&f, &md5, &sha1, OPT::Added, CLASS::File)?;
            let log_str = format!("File added: {}", f);
            log_to_file(&log_str);
        } else {
            // database have such file
            let files = &r[0];
            if files.last_opt == OPT::Deleted.to_string() {
                db.insert(&f, &md5, &sha1, OPT::Added, CLASS::File)?;
                let log_str = format!("File added: {}", f);
                log_to_file(&log_str);
            } else if files.md5 != md5 || files.sha1 != sha1 {
                db.insert(&f, &md5, &sha1, OPT::Changed, CLASS::File)?;
                let log_str = format!("File changed: {}", f);
                log_to_file(&log_str);
            }
        }
    }

    for f in disk_floders {
        let r = db.select(&f)?;
        let md5 = String::new();
        let sha1 = String::new();

        if r.len() == 0 {
            // database no such file
            db.insert(&f, &md5, &sha1, OPT::Added, CLASS::Folder)?;
            let log_str = format!("Floder added: {}", f);
            log_to_file(&log_str);
        } else {
            // database have such file
            let files = &r[0];
            if files.last_opt == OPT::Deleted.to_string() {
                db.insert(&f, &md5, &sha1, OPT::Added, CLASS::Folder)?;
                let log_str = format!("Floder added: {}", f);
                log_to_file(&log_str);
            }
        }
    }
    Ok(())
}

/// Travel all database stored files to find file delete
fn travel_database(db: &WatchDB, watchpath: &str) -> Result<(), rusqlite::Error> {
    let db_files = db.select_distinct()?;
    let (disk_files, disk_folders) = disk_files_and_folders(watchpath);

    for dfs in db_files {
        let path = dfs.path;
        let opt = dfs.last_opt;
        let class = dfs.class;
        if class == CLASS::File.to_string()
            && !disk_files.contains(&path)
            && opt != OPT::Deleted.to_string()
        {
            let data = vec![];
            let md5 = md5_cal(&data);
            let sha1 = sha1_cal(&data);
            db.insert(&path, &md5, &sha1, OPT::Deleted, CLASS::File)?;
            let log_str = format!("File deleted: {}", path);
            log_to_file(&log_str);
        }
        if class == CLASS::Folder.to_string()
            && !disk_folders.contains(&path)
            && opt != OPT::Deleted.to_string()
        {
            let md5 = String::new();
            let sha1 = String::new();
            db.insert(&path, &md5, &sha1, OPT::Deleted, CLASS::Folder)?;
            let log_str = format!("Folder deleted: {}", path);
            log_to_file(&log_str);
        }
    }
    Ok(())
}

fn print_db(db: &WatchDB) -> Result<(), rusqlite::Error> {
    let db_files = db.select_all()?;
    if db_files.len() > 0 {
        println!("===");
        for dfs in db_files {
            println!("path: {}", dfs.path);
            println!("md5: {}", dfs.md5);
            println!("sha1: {}", dfs.sha1);
            println!("opt: {}", dfs.last_opt);
            println!(
                "time: {}-{}-{} {}:{}:{}",
                dfs.time.year(),
                dfs.time.month(),
                dfs.time.day(),
                dfs.time.hour(),
                dfs.time.minute(),
                dfs.time.second()
            );
            println!("===");
        }
    }
    Ok(())
}

fn main() -> Result<(), rusqlite::Error> {
    let args = Args::parse();
    let db = WatchDB::connect()?;

    if args.printdb {
        print_db(&db)?;
    } else {
        println!("diskwacher runing...");
        if args.flush {
            db.flush_all()?;
        }
        loop {
            travel_disk(&db, &args.watch)?;
            travel_database(&db, &args.watch)?;
            // sleep
            let ten_millis = time::Duration::from_secs_f32(0.5);
            thread::sleep(ten_millis);
        }
    }
    Ok(())
}
