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
            let tx = self.conn.transaction().unwrap();
            tx.execute(
                "INSERT OR REPLACE INTO Recents(path) VALUES(?);",
                [recent.path.to_string()],
            )
            .expect("Error inserting Recent into Database");
            tx.commit().expect("Error commiting changes to DB");
        }

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
                icon BLOB
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
