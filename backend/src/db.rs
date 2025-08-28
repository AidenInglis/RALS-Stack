use anyhow::Result;
use chrono::{Utc, Duration};
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

// ---------- Users ----------

#[derive(Clone)]
pub struct DbUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: i64,
}

pub async fn pool(dsn: &str) -> Result<SqlitePool> {
    Ok(SqlitePool::connect(dsn).await?)
}

pub async fn find_user_by_email(pool: &SqlitePool, email: &str) -> Result<Option<DbUser>> {
    let row = sqlx::query(
        "SELECT id,email,password_hash,is_admin,created_at FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| DbUser {
        id: r.get("id"),
        email: r.get("email"),
        password_hash: r.get("password_hash"),
        is_admin: r.get::<i64,_>("is_admin") == 1,
        created_at: r.get("created_at"),
    }))
}

pub async fn first_user_exists(pool: &SqlitePool) -> Result<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(pool).await?;
    Ok(count > 0)
}

pub async fn create_user(pool: &SqlitePool, email: &str, password_hash: &str, is_admin: bool) -> Result<DbUser> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().timestamp();
    let is_admin_i = if is_admin { 1 } else { 0 };

    sqlx::query("INSERT INTO users(id,email,password_hash,is_admin,created_at) VALUES(?,?,?,?,?)")
        .bind(&id)
        .bind(email)
        .bind(password_hash)
        .bind(is_admin_i)
        .bind(created_at)
        .execute(pool)
        .await?;

    Ok(DbUser {
        id,
        email: email.to_string(),
        password_hash: password_hash.to_string(),
        is_admin,
        created_at,
    })
}

pub async fn is_user_admin(pool: &SqlitePool, user_id: &str) -> Result<bool> {
    let one: Option<i64> = sqlx::query_scalar("SELECT 1 FROM users WHERE id=? AND is_admin=1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(one.is_some())
}

// ---------- Coupons ----------

#[derive(Clone)]
pub struct DbCoupon {
    pub id: String,
    pub code: String,
    pub description: String,
    pub service: String,
    pub expires_at: i64,          // unix secs
    pub owner_id: Option<String>, // nullable
    pub created_at: i64,
}

pub async fn create_coupon(
    pool: &SqlitePool,
    code: &str,
    description: &str,
    service: &str,
    expires_in_days: i64,
    owner_id: Option<&str>,
) -> Result<DbCoupon> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = (now + Duration::days(expires_in_days)).timestamp();

    sqlx::query("INSERT INTO coupons(id,code,description,service,expires_at,owner_id,created_at)
                 VALUES(?,?,?,?,?,?,?)")
        .bind(&id)
        .bind(code)
        .bind(description)
        .bind(service)
        .bind(expires_at)
        .bind(owner_id)
        .bind(now.timestamp())
        .execute(pool)
        .await?;

    Ok(DbCoupon {
        id,
        code: code.to_string(),
        description: description.to_string(),
        service: service.to_string(),
        expires_at,
        owner_id: owner_id.map(|s| s.to_string()),
        created_at: now.timestamp(),
    })
}

pub async fn update_coupon_by_code(
    pool: &SqlitePool,
    code: &str,
    description: Option<&str>,
    service: Option<&str>,
    expires_in_days: Option<i64>,
    owner_id: Option<Option<&str>>, // Some(Some(x)) set, Some(None) clear, None leave unchanged
) -> Result<bool> {
    // Fetch existing
    let Some(cur) = get_coupon_by_code(pool, code).await? else { return Ok(false); };

    let new_desc = description.unwrap_or(&cur.description);
    let new_serv = service.unwrap_or(&cur.service);
    let new_expires_at = if let Some(days) = expires_in_days {
        (Utc::now() + Duration::days(days)).timestamp()
    } else {
        cur.expires_at
    };
    let new_owner: Option<&str> = match owner_id {
        None => cur.owner_id.as_deref(),
        Some(Some(v)) => Some(v),
        Some(None) => None,
    };

    let n = sqlx::query("UPDATE coupons SET description=?, service=?, expires_at=?, owner_id=? WHERE code=?")
        .bind(new_desc)
        .bind(new_serv)
        .bind(new_expires_at)
        .bind(new_owner)
        .bind(code)
        .execute(pool)
        .await?
        .rows_affected();

    Ok(n == 1)
}

pub async fn delete_coupon_by_code(pool: &SqlitePool, code: &str) -> Result<bool> {
    let n = sqlx::query("DELETE FROM coupons WHERE code=?")
        .bind(code)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n == 1)
}

pub async fn get_coupon_by_code(pool: &SqlitePool, code: &str) -> Result<Option<DbCoupon>> {
    let row = sqlx::query("SELECT id,code,description,service,expires_at,owner_id,created_at FROM coupons WHERE code=?")
        .bind(code)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|r| DbCoupon {
        id: r.get("id"),
        code: r.get("code"),
        description: r.get("description"),
        service: r.get("service"),
        expires_at: r.get("expires_at"),
        owner_id: r.get::<Option<String>,_>("owner_id"),
        created_at: r.get("created_at"),
    }))
}

pub async fn list_coupons(pool: &SqlitePool, active_only: bool) -> Result<Vec<DbCoupon>> {
    let rows = if active_only {
        sqlx::query("SELECT id,code,description,service,expires_at,owner_id,created_at
                     FROM coupons WHERE expires_at > ? ORDER BY created_at DESC")
            .bind(Utc::now().timestamp())
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query("SELECT id,code,description,service,expires_at,owner_id,created_at
                     FROM coupons ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?
    };

    Ok(rows
        .into_iter()
        .map(|r| DbCoupon {
            id: r.get("id"),
            code: r.get("code"),
            description: r.get("description"),
            service: r.get("service"),
            expires_at: r.get("expires_at"),
            owner_id: r.get::<Option<String>,_>("owner_id"),
            created_at: r.get("created_at"),
        })
        .collect())
}

// User claims an unowned, non-expired coupon.
// Returns the coupon if claim succeeded, or Ok(None) if it was already owned/expired/not found.
pub async fn claim_coupon(pool: &SqlitePool, code: &str, user_id: &str) -> Result<Option<DbCoupon>> {
    let now = Utc::now().timestamp();
    let n = sqlx::query(
        "UPDATE coupons SET owner_id=? WHERE code=? AND owner_id IS NULL AND expires_at > ?"
    )
    .bind(user_id)
    .bind(code)
    .bind(now)
    .execute(pool)
    .await?
    .rows_affected();

    if n == 1 {
        get_coupon_by_code(pool, code).await
    } else {
        Ok(None)
    }
}

// User releases a coupon they own.
pub async fn release_coupon(pool: &SqlitePool, code: &str, user_id: &str) -> Result<bool> {
    let n = sqlx::query("UPDATE coupons SET owner_id=NULL WHERE code=? AND owner_id=?")
        .bind(code)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(n == 1)
}