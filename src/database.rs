use std::path::Path;

use raylib::prelude::Rectangle;
use rusqlite::{Connection, Error, Row};

use crate::structs::{Chunk, ComicMetadata};

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

        rows.mapped(sqlite_row_to_metadata)
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
            eprintln!("Inserting {md:?} into DB");
            tx.execute(
                "INSERT OR REPLACE INTO Metadata VALUES(?,?,?,?)",
                [
                    md.path.to_string(),
                    md.chunk_count.to_string(),
                    md.last_seen_chunk.to_string(),
                    String::new(),
                ],
            )
            .expect("Error inserting metadata into transaction");
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

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS
            Chunks(
                path TEXT,
                x INTEGER,
                y INTEGER,
                w INTEGER,
                h INTEGER,
                texture_index INTEGER,
                CONSTRAINT \"uniq\" UNIQUE (\"path\", \"texture_index\", \"y\") ON CONFLICT IGNORE
            );",
            [],
        )
        .expect("Error creating recents table");

        Self { conn }
    }

    pub fn metadata_for(&self, path: &str) -> Option<ComicMetadata> {
        match self.conn.query_row(
            "SELECT * FROM Metadata WHERE Path==? LIMIT 1;",
            [path],
            sqlite_row_to_metadata,
        ) {
            Ok(metadata) => {
                eprintln!("Got metadata for {path}: {metadata:?}");
                return Some(metadata);
            }
            Err(error) => {
                eprintln!("Error getting metadata: {:?}", error);
                return None;
            }
        }
    }

    pub fn chunks_for(&self, path: &str) -> Vec<Chunk> {
        if let Ok(mut stmt) = self
            .conn
            .prepare("SELECT x,y,w,h,texture_index FROM Chunks WHERE Path==?;")
        {
            if let Ok(results) = stmt.query([path]) {
                return results
                    .mapped(sqlite_row_to_chunk)
                    .filter(|x| x.is_ok())
                    .map(|x| x.unwrap())
                    .collect::<Vec<Chunk>>();
            }
        }

        Vec::new()
    }

    pub fn save_chunk_cache(&mut self, path: String, all_chunks: Vec<Chunk>) {
        let mut tx = self
            .conn
            .transaction()
            .expect("Couldn't start transaction to save chunks!");

        if let Ok(mut stmt) = tx.prepare("INSERT INTO Chunks VALUES(?,?,?,?,?,?);") {
            for c in all_chunks {
                stmt.execute([
                    &path,
                    &c.rect.x.to_string(),
                    &c.rect.y.to_string(),
                    &c.rect.width.to_string(),
                    &c.rect.height.to_string(),
                    &c.texture_index.to_string(),
                ])
                .expect("Error inserting Chunk row into db");
            }
        }else{
            println!("Couldn't prepare statement to insert chunks into DB");
        }

        tx.commit();
    }
}

fn sqlite_row_to_chunk(row: &Row) -> Result<Chunk, Error> {
    let texture_index: usize = row.get(4).unwrap();

    Ok(Chunk {
        rect: Rectangle::new(
            row.get(0).unwrap(),
            row.get(1).unwrap(),
            row.get(2).unwrap(),
            row.get(3).unwrap(),
        ),
        texture_index,
    })
}

fn sqlite_row_to_metadata(row: &Row) -> Result<ComicMetadata, Error> {
    let path: String = row.get(0)?;
    let chunk_count: usize = row.get(1)?;
    let last_seen_chunk: usize = row.get(2)?;

    let path_object = Path::new(path.as_str());
    let title = String::from(path_object.file_name().unwrap().to_str().unwrap());

    eprintln!("ROW: {path} {title} {chunk_count} {last_seen_chunk}");

    Ok(ComicMetadata {
        title,
        chunk_count,
        last_seen_chunk,
        path,
        thumbnail: None,
    })
}
