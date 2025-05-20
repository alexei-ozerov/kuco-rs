use color_eyre::eyre::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

use std::env;
use std::fs;
use std::path::PathBuf;
use std::{path::Path, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new() -> Result<Self> {
        let home_path;

        // TODO: remove this ... won't work on Windows
        match env::home_dir() {
            Some(home) => {
                home_path = home;
            }
            None => {
                home_path = PathBuf::new();
            }
        }

        let sqlite_store_path = format!("{}/.kuco/sqlite", home_path.to_str().unwrap());
        let path = Path::new(&sqlite_store_path);

        if !path.exists() {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
        }

        let timeout: f64 = 10.0;

        let opts = SqliteConnectOptions::from_str(path.as_os_str().to_str().unwrap())?
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs_f64(timeout))
            .connect_with(opts)
            .await?;

        Self::setup_db(&pool).await?;

        Ok(Self { pool })
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        // sqlx::migrate!("./record-migrations").run(pool).await?;

        Ok(())
    }
}
