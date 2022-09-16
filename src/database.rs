use std::{default, path::Path};

use rusqlite::{Connection, Error, Row};

use crate::structs::ComicMetadata;

pub struct Database {
    pub conn: Connection,
}

#[allow(dead_code, unused)]
impl Database {
    pub fn get_recents(&mut self) -> Vec<ComicMetadata> {
        let mut query = self
            .conn
            .prepare(
                "
                        SELECT
                            Metadata.*
                        FROM
                            Recents
                            LEFT JOIN Metadata ON Recents.path = Metadata.path
                            LIMIT 8;",
            )
            .expect("Error getting recent list from DB");

        let mut rows = query.query([]).expect("Error getting rows from DB");

        let transform_row = |row: &Row| -> Result<ComicMetadata, Error> {
            let path: String = row.get(0)?;
            let chunk_count: usize = row.get(1)?;
            let last_seen_chunk: usize = row.get(2)?;

            let path_object = Path::new(path.as_str());
            let title = String::from(path_object.to_str().unwrap());

            println!("ROW: {path} {chunk_count} {title}");

            Ok(ComicMetadata {
                title,
                chunk_count,
                last_seen_chunk,
                path,
                thumbnail: None,
            })
        };

        rows.mapped(|row| transform_row(row))
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect()
    }

    pub fn save_recents(&mut self, recents: &Vec<ComicMetadata>) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM Recents;", [])
            .expect("Error clearing recents table");

        for recent in recents.iter() {
            let tx = self
                .conn
                .transaction()
                .expect("Couldn't start a transation");
            tx.execute(
                "INSERT OR REPLACE INTO Recents(path) VALUES(?);",
                [recent.path.to_string()],
            )
            .expect("Error inserting Recent into Database");
            tx.commit().expect("Error writing changes to DB");
        }

        Ok(())
    }

    pub fn save_metadata(&mut self, metadata: &Vec<&ComicMetadata>) -> Result<(), rusqlite::Error> {
        let tx = self
            .conn
            .transaction()
            .expect("Couldn't start a transation");

        for md in metadata.iter() {
            println!("Inserting {md:?} into DB");
            tx.execute("INSERT OR REPLACE INTO Metadata VALUES(?,?,?,?)", [
                md.path.to_string(),
                md.chunk_count.to_string(),
                md.last_seen_chunk.to_string(),
                String::new(),
            ]).expect("Error inserting metadata into transaction");
        }
        tx.commit()?;

        Ok(())
    }

    pub fn new() -> Self {
        let conn = Connection::open("metadata.sqlite3").expect("Couldn't open metadata database");

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS
            Metadata(
                path TEXT PRIMARY KEY,
                chunk_count INTEGER,
                last_chunk INTEGER,
                icon TEXT
            )",
            [],
        )
        .expect("Error creating metadata table");

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS
            Recents(
                path TEXT UNIQUE
            );",
            [],
        )
        .expect("Error creating recents table");

        Self { conn }
    }
}
