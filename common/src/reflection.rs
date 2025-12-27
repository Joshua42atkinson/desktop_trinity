use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use chrono::Utc;
#[cfg(feature = "ssr")]
use sqlx::PgPool;

#[derive(Serialize, Deserialize, Clone)]
pub struct ReflectionEntry {
    pub user_id: i64,
    pub challenge_name: String,
    pub reflection_text: String,
}

#[cfg(feature = "ssr")]
pub async fn save_reflection_entry(
    pool: &PgPool,
    user_id: i64,
    challenge_name: &str,
    reflection_text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO reflection_entries (user_id, challenge_name, reflection_text, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        challenge_name,
        reflection_text,
        Utc::now()
    )
    .execute(pool)
    .await?;
    Ok(())
}
