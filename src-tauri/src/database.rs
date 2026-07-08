use sqlx::MySqlPool;
use std::env;

pub struct Database {
    pool: MySqlPool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct User {
    pub id: i64,
    pub discord_id: String,
    pub username: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        dotenv::dotenv().ok();

        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            env::var("MYSQL_USER").expect("MYSQL_USER must be set"),
            env::var("MYSQL_PASSWORD").expect("MYSQL_PASSWORD must be set"),
            env::var("MYSQL_HOST").expect("MYSQL_HOST must be set"),
            env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string()),
            env::var("MYSQL_DATABASE").expect("MYSQL_DATABASE must be set")
        );

        let pool = MySqlPool::connect(&database_url).await?;

        Ok(Self { pool })
    }

    pub async fn create_tables(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INT AUTO_INCREMENT PRIMARY KEY,
                discord_id VARCHAR(255) UNIQUE NOT NULL,
                username VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_by_discord_id(&self, discord_id: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, discord_id, username, created_at FROM users WHERE discord_id = ?",
        )
        .bind(discord_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, discord_id, username, created_at)| User {
            id,
            discord_id,
            username,
            created_at,
        }))
    }

    pub async fn create_user(&self, discord_id: &str, username: &str) -> Result<User, sqlx::Error> {
        let result = sqlx::query("INSERT INTO users (discord_id, username) VALUES (?, ?)")
            .bind(discord_id)
            .bind(username)
            .execute(&self.pool)
            .await?;

        let id = result.last_insert_id() as i64;

        Ok(User {
            id,
            discord_id: discord_id.to_string(),
            username: username.to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn update_username(&self, discord_id: &str, username: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET username = ? WHERE discord_id = ?")
            .bind(username)
            .bind(discord_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
