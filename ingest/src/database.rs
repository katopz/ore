use crate::types::RoundWinner;
use anyhow::Context;
use anyhow::Result;
use turso::{Builder, Connection};

/// Database operations module
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Initialize database connection
    pub async fn new(db_url: &str) -> Result<Self> {
        let db = Builder::new_local(db_url)
            .build()
            .await
            .context("Failed to initialize Turso database")?;

        let conn = db
            .connect()
            .context("Failed to connect to Turso database")?;

        Ok(Self { conn })
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        self.conn
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS round_winners (
                id INTEGER PRIMARY KEY,
                address TEXT NOT NULL,
                winning_square INTEGER NOT NULL,
                winning_row INTEGER NOT NULL,
                winning_col INTEGER NOT NULL,
                top_miner TEXT NOT NULL,
                top_miner_reward INTEGER NOT NULL,
                split_reward BOOLEAN NOT NULL,
                motherlode_hit BOOLEAN NOT NULL,
                motherlode_amount INTEGER NOT NULL,
                total_deployed INTEGER NOT NULL,
                total_vaulted INTEGER NOT NULL,
                total_winnings INTEGER NOT NULL,
                winners_count INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )"#,
                (),
            )
            .await?;

        // Create index for faster queries
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_round_winners_id ON round_winners(id)",
                (),
            )
            .await?;

        Ok(())
    }

    /// Get the last processed round ID from database
    pub async fn get_last_round_id(&self) -> Result<Option<i64>> {
        let mut rows = self
            .conn
            .query("SELECT id FROM round_winners ORDER BY id DESC LIMIT 1", ())
            .await?;

        if let Some(row) = rows.next().await? {
            let id: i64 = row.get(0)?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    /// Check if round already exists in database
    pub async fn round_exists(&self, round_id: i64) -> Result<bool> {
        let mut rows = self
            .conn
            .query("SELECT id FROM round_winners WHERE id = ?", [round_id])
            .await?;

        Ok(rows.next().await?.is_some())
    }

    /// Save round winner data to database
    pub async fn save_round_winner(&self, winner: &RoundWinner) -> Result<()> {
        self.conn
            .execute(
                r#"
                INSERT INTO round_winners (
                    id, address, winning_square, winning_row, winning_col,
                    top_miner, top_miner_reward, split_reward, motherlode_hit,
                    motherlode_amount, total_deployed, total_vaulted, total_winnings,
                    winners_count, expires_at, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                (
                    winner.id,
                    winner.address.as_str(),
                    winner.winning_square,
                    winner.winning_row,
                    winner.winning_col,
                    winner.top_miner.as_str(),
                    winner.top_miner_reward,
                    winner.split_reward,
                    winner.motherlode_hit,
                    winner.motherlode_amount,
                    winner.total_deployed,
                    winner.total_vaulted,
                    winner.total_winnings,
                    winner.winners_count,
                    winner.expires_at,
                    winner.created_at.to_rfc3339(),
                ),
            )
            .await?;

        Ok(())
    }

    /// List rounds with pagination
    pub async fn list_rounds(&self, limit: Option<i64>) -> Result<Vec<RoundWinner>> {
        let limit = limit.unwrap_or(100); // Default to 100

        let mut rows = self
            .conn
            .query(
                "SELECT id, address, winning_square, winning_row, winning_col,
                    top_miner, top_miner_reward, split_reward, motherlode_hit,
                    motherlode_amount, total_deployed, total_vaulted, total_winnings,
                    winners_count, expires_at, created_at
             FROM round_winners
             ORDER BY id DESC
             LIMIT ?",
                [limit],
            )
            .await?;

        let mut rounds = Vec::new();
        while let Some(row) = rows.next().await? {
            let created_at_str: String = row.get(15)?;
            rounds.push(RoundWinner {
                id: row.get(0)?,
                address: row.get(1)?,
                winning_square: row.get(2)?,
                winning_row: row.get(3)?,
                winning_col: row.get(4)?,
                top_miner: row.get(5)?,
                top_miner_reward: row.get(6)?,
                split_reward: row.get(7)?,
                motherlode_hit: row.get(8)?,
                motherlode_amount: row.get(9)?,
                total_deployed: row.get(10)?,
                total_vaulted: row.get(11)?,
                total_winnings: row.get(12)?,
                winners_count: row.get(13)?,
                expires_at: row.get(14)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .context("Failed to parse created_at")?
                    .with_timezone(&chrono::Utc),
            });
        }

        Ok(rounds)
    }

    /// Get total count of rounds in database
    pub async fn get_rounds_count(&self) -> Result<i64> {
        let mut rows = self
            .conn
            .query("SELECT COUNT(*) FROM round_winners", ())
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(row.get(0)?)
        } else {
            Ok(0)
        }
    }
}
