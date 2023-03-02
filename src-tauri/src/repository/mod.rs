use directories::ProjectDirs;
use rusqlite::Connection;
use std::{fs, path};

use crate::errors::{ControlledErrors, Error, Result};
use crate::repository::clickhouse_connection::ClickhouseConnection;

pub mod clickhouse_connection;

const TABLE: &str = "connections";
const DATABASE_NAME: &str = "clickmate.db";
const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "clickmate";
const APPLICATION: &str = "clickmate";

pub struct Repository {
    connection: Connection,
}

impl Repository {
    pub fn new(passphrase: &String) -> Result<Repository> {
        let database_file = check_database_file(&false)?;
        let connection = Connection::open(&database_file)?;

        connection.pragma_update(None, "KEY", passphrase)?;

        connection.execute(
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT,
                    host TEXT NOT NULL,
                    port INTEGER NOT NULL,
                    secure BOOLEAN NOT NULL,
                    username TEXT NOT NULL,
                    password TEXT,
                    database TEXT
                )",
                TABLE
            )
            .as_str(),
            [],
        )?;

        Ok(Repository { connection })
    }

    pub fn create(&self, connection: ClickhouseConnection) -> Result<()> {
        self.connection.execute(
            format!("INSERT INTO {} (name, host, port, secure, username, password, database) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", TABLE).as_str(),
            (connection.name, connection.host, connection.port, connection.secure, connection.username, connection.password, connection.database)
        )?;

        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<ClickhouseConnection>> {
        let mut stmt = self.connection.prepare(
            format!(
                "SELECT id, name, host, port, secure, username, password, database FROM {}",
                TABLE
            )
            .as_str(),
        )?;
        let connections_iter =
            stmt.query_map([], |row| Ok(map_row_to_clickhouse_connection(row)))?;

        let mut connections = Vec::new();
        for connection in connections_iter {
            connections.push(connection?);
        }

        Ok(connections)
    }

    pub fn get_by_id(&self, id: i32) -> Result<ClickhouseConnection> {
        let mut stmt = self
            .connection
            .prepare(format!("SELECT id, name, host, port, secure, username, password, database FROM {} WHERE id = ?1", TABLE).as_str())?;
        let connection = stmt.query_row([id], |row| Ok(map_row_to_clickhouse_connection(row)))?;

        Ok(connection)
    }

    pub fn update(&self, id: u32, connection: ClickhouseConnection) -> Result<()> {
        self.connection.execute(
            format!("UPDATE {} SET name = ?1, host = ?2, port = ?3, secure = ?4, username = ?5, password = ?6, database = ?7 WHERE id = ?8", TABLE).as_str(),
            (connection.name, connection.host, connection.port, connection.secure, connection.username, connection.password, connection.database, id)
        )?;

        Ok(())
    }

    pub fn delete(&self, id: u32) -> Result<()> {
        self.connection.execute(
            format!("DELETE FROM {} WHERE id = ?1", TABLE).as_str(),
            [id],
        )?;

        Ok(())
    }
}

fn map_row_to_clickhouse_connection(row: &rusqlite::Row) -> ClickhouseConnection {
    ClickhouseConnection {
        id: row.get(0).unwrap_or(None),
        name: row.get(1).unwrap_or(None),
        host: row.get(2).unwrap(),
        port: row.get(3).unwrap(),
        secure: row.get(4).unwrap(),
        username: row.get(5).unwrap(),
        password: row.get(6).unwrap_or(None),
        database: row.get(7).unwrap_or(None),
    }
}

pub fn check_database_file(for_first_time: &bool) -> core::result::Result<String, Error> {
    let option_project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION);
    if let None = option_project_dirs {
        return Err(Error::ControlledError(ControlledErrors::NoProjectDirectory));
    }

    let project_dirs = option_project_dirs.unwrap();
    let data_dir = project_dirs.data_dir().to_str();
    if let None = data_dir {
        return Err(Error::ControlledError(ControlledErrors::NoDataDirectory));
    }
    let data_dir = data_dir.unwrap();

    let is_path_data_dir = path::Path::new(&data_dir).is_dir();
    if !is_path_data_dir {
        if let Err(e) = fs::create_dir_all(&data_dir) {
            return Err(Error::IOError(e));
        }
    }

    let database_file = format!("{}/{}", &data_dir, DATABASE_NAME);
    let is_path_database_file = path::Path::new(&database_file).is_file();
    if !is_path_database_file & for_first_time {
        return Err(Error::ControlledError(ControlledErrors::FirstTime));
    }

    Ok(database_file)
}
