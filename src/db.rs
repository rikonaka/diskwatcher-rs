use chrono::DateTime;
use chrono::Local;
use rusqlite;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::Result;
use std::fmt;

use super::DB_FILE;

const TABLE_NAME: &str = "data";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CLASS {
    File,
    Folder,
}

impl fmt::Display for CLASS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OPT {
    Added,
    Changed,
    Deleted,
}

impl fmt::Display for OPT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Watch {
    pub path: String,
    pub md5: String,
    pub sha1: String,
    pub last_opt: String,
    pub time: DateTime<Local>,
    pub class: String,
}

pub struct WatchDB {
    connection: Connection,
}

impl WatchDB {
    pub fn create_table(&self) -> Result<usize, rusqlite::Error> {
        let sql_str = format!("CREATE TABLE IF NOT EXISTS {TABLE_NAME} (path TEXT, md5 TEXT, sha1 TEXT, last_opt TEXT, time DATETIME, class TEXT)");
        self.connection.execute(&sql_str, params![])
    }
    pub fn drop_table(&self) -> Result<usize, rusqlite::Error> {
        let sql_str = format!("DROP TABLE IF EXISTS {TABLE_NAME}");
        self.connection.execute(&sql_str, params![])
    }
    pub fn connect() -> Result<WatchDB, rusqlite::Error> {
        let connection = Connection::open(DB_FILE).unwrap();
        let db = WatchDB { connection };
        db.create_table()?;
        Ok(db)
    }
    pub fn insert(
        &self,
        path: &str,
        md5: &str,
        sha1: &str,
        last_opt: OPT,
        class: CLASS,
    ) -> Result<usize, rusqlite::Error> {
        let time = Local::now();
        let last_opt = last_opt.to_string();
        let class = class.to_string();
        let sql_str = format!(
            "INSERT INTO {TABLE_NAME} (path, md5, sha1, last_opt, time, class) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        );
        self.connection
            .execute(&sql_str, params![path, md5, sha1, last_opt, time, class])
    }
    pub fn select(&self, path: &str) -> Result<Vec<Watch>, rusqlite::Error> {
        let sql_str =
            format!("SELECT * FROM {TABLE_NAME} WHERE path='{path}' ORDER BY time DESC LIMIT 1");
        let mut stmt = self.connection.prepare(&sql_str)?;
        let files_iter = stmt.query_map([], |row| {
            Ok(Watch {
                path: row.get(0)?,
                md5: row.get(1)?,
                sha1: row.get(2)?,
                last_opt: row.get(3)?,
                time: row.get(4)?,
                class: row.get(5)?,
            })
        })?;

        let mut ret = Vec::new();
        for f in files_iter {
            ret.push(f?);
        }

        Ok(ret)
    }
    pub fn select_distinct(&self) -> Result<Vec<Watch>, rusqlite::Error> {
        let sql_str = format!("SELECT * FROM {} ORDER BY time DESC", TABLE_NAME);
        let mut stmt = self.connection.prepare(&sql_str)?;
        let files_iter = stmt.query_map([], |row| {
            Ok(Watch {
                path: row.get(0)?,
                md5: row.get(1)?,
                sha1: row.get(2)?,
                last_opt: row.get(3)?,
                time: row.get(4)?,
                class: row.get(5)?,
            })
        })?;

        let mut distinct_path = Vec::new();
        let mut ret = Vec::new();
        for f in files_iter {
            let f = f?;
            if !distinct_path.contains(&f.path) {
                distinct_path.push(f.path.to_string());
                ret.push(f);
            }
        }
        Ok(ret)
    }
    pub fn select_all(&self) -> Result<Vec<Watch>, rusqlite::Error> {
        let sql_str = format!("SELECT * FROM {} ORDER BY time ASC", TABLE_NAME);
        let mut stmt = self.connection.prepare(&sql_str)?;
        let files_iter = stmt.query_map([], |row| {
            Ok(Watch {
                path: row.get(0)?,
                md5: row.get(1)?,
                sha1: row.get(2)?,
                last_opt: row.get(3)?,
                time: row.get(4)?,
                class: row.get(5)?,
            })
        })?;

        let mut ret = Vec::new();
        for f in files_iter {
            ret.push(f?);
        }
        Ok(ret)
    }
    pub fn flush_all(&self) -> Result<(), rusqlite::Error> {
        self.drop_table()?;
        self.create_table()?;
        Ok(())
    }
}
