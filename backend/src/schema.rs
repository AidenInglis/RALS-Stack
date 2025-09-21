use async_graphql::{
    Context, Object, Schema, EmptySubscription, Result as GqlResult, SimpleObject, InputObject,
};
use sqlx::{Row, SqlitePool};

use crate::{auth, db};

// ---------- App State ----------
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_secret: String,
}

// ---------- GraphQL Types ----------
#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")] // you preferred snake_case in the UI
pub struct User {
    pub id: String,
    pub email: String,
    pub is_admin: bool,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "snake_case")]
pub struct Coupon {
    pub id: String,
    pub code: String,
    pub description: String,
    pub service: String,
    pub expires_at: i64,           // unix seconds
    pub owner_id: Option<String>,  // nullable
    pub created_at: i64,
}

// ---------- Inputs ----------
#[derive(InputObject)]
pub struct RegisterInput { pub email: String, pub password: String }

#[derive(InputObject)]
pub struct LoginInput { pub email: String, pub password: String }

#[derive(InputObject)]
pub struct CreateCouponInput {
    pub code: String,
    pub description: String,
    pub service: String,
    /// How many days from now it should expire
    pub expires_in_days: i64,
    /// Optional: assign to a user id at creation
    pub owner_id: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateCouponInput {
    /// Coupon to update (by code)
    pub code: String,
    /// Optional updates; omit to leave unchanged
    pub description: Option<String>,
    pub service: Option<String>,
    pub expires_in_days: Option<i64>,
    /// Set new owner (takes precedence if provided)
    pub owner_id: Option<String>,
    /// If true and owner_id not provided, clear owner
    pub clear_owner: Option<bool>,
}

// ---------- Schema ----------
pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str { "ok" }

    async fn me(&self, ctx: &Context<'_>) -> GqlResult<Option<User>> {
        let st = ctx.data_unchecked::<AppState>();
        if let Some(uid) = user_id_from_headers(ctx, &st.jwt_secret)? {
            let row = sqlx::query("SELECT id,email,is_admin FROM users WHERE id = ?")
                .bind(&uid)
                .fetch_optional(&st.pool)
                .await?;
            if let Some(r) = row {
                return Ok(Some(User {
                    id: r.get("id"),
                    email: r.get("email"),
                    is_admin: r.get::<i64,_>("is_admin") == 1,
                }));
            }
        }
        Ok(None)
    }

    /// Public list of coupons. `active_only` defaults to true.
    async fn list_coupons(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = true)] active_only: bool,
    ) -> GqlResult<Vec<Coupon>> {
        let st = ctx.data_unchecked::<AppState>();
        let rows = db::list_coupons(&st.pool, active_only).await?;
        Ok(rows.into_iter().map(db_coupon_to_gql).collect())
    }

    /// Optional helper: fetch a single coupon by code
    async fn get_coupon(
        &self,
        ctx: &Context<'_>,
        code: String,
    ) -> GqlResult<Option<Coupon>> {
        let st = ctx.data_unchecked::<AppState>();
        Ok(db::get_coupon_by_code(&st.pool, &code).await?.map(db_coupon_to_gql))
    }
    async fn my_coupons(&self, ctx: &Context<'_>) -> GqlResult<Vec<Coupon>> {
        use sqlx::Row;
        let st = ctx.data_unchecked::<AppState>();
        let uid = require_user(ctx, &st.jwt_secret)?;
        let now = chrono::Utc::now().timestamp();
        let rows = sqlx::query(
            "SELECT id,code,description,service,expires_at,owner_id,created_at
             FROM coupons WHERE owner_id = ? AND expires_at > ? ORDER BY created_at DESC"
        )
        .bind(uid)
        .bind(now)
        .fetch_all(&st.pool)
        .await?;

        Ok(rows.into_iter().map(|r| Coupon {
            id: r.get("id"),
            code: r.get("code"),
            description: r.get("description"),
            service: r.get("service"),
            expires_at: r.get("expires_at"),
            owner_id: r.get::<Option<String>,_>("owner_id"),
            created_at: r.get("created_at"),
        }).collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // -------- Auth --------
    async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> GqlResult<User> {
        let st = ctx.data_unchecked::<AppState>();

        if let Some(u) = db::find_user_by_email(&st.pool, &input.email).await? {
            return Ok(User { id: u.id, email: u.email, is_admin: u.is_admin });
        }

        let hash = auth::hash_password(&input.password)?;
        // First-ever user becomes admin (bootstrap)
        let is_first = !db::first_user_exists(&st.pool).await?;
        let u = db::create_user(&st.pool, &input.email, &hash, is_first).await?;

        Ok(User { id: u.id, email: u.email, is_admin: u.is_admin })
    }

    /// Claim an unowned, non-expired coupon for the current user.
    async fn claim_coupon(&self, ctx: &Context<'_>, code: String) -> GqlResult<Option<Coupon>> {
        let st = ctx.data_unchecked::<AppState>();
        let uid = require_user(ctx, &st.jwt_secret)?;
        let claimed = db::claim_coupon(&st.pool, &code, &uid).await?;
        Ok(claimed.map(db_coupon_to_gql))
    }

    /// Release a coupon currently owned by the user.
    async fn release_coupon(&self, ctx: &Context<'_>, code: String) -> GqlResult<bool> {
        let st = ctx.data_unchecked::<AppState>();
        let uid = require_user(ctx, &st.jwt_secret)?;
        Ok(db::release_coupon(&st.pool, &code, &uid).await?)
    }

    async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> GqlResult<String> {
        let st = ctx.data_unchecked::<AppState>();
        let Some(u) = db::find_user_by_email(&st.pool, &input.email).await? else {
            return Ok("".into());
        };
        if !auth::verify_password(&u.password_hash, &input.password) {
            return Ok("".into());
        }
        let token = auth::make_jwt_3min(&st.jwt_secret, &u.id)?;
        Ok(token)
    }

    // -------- Admin: Coupon CRUD --------
    async fn create_coupon(&self, ctx: &Context<'_>, input: CreateCouponInput) -> GqlResult<Coupon> {
        let st = ctx.data_unchecked::<AppState>();
        require_admin(ctx, &st.pool, &st.jwt_secret).await?;

        let created = db::create_coupon(
            &st.pool,
            &input.code,
            &input.description,
            &input.service,
            input.expires_in_days,
            input.owner_id.as_deref(),
        ).await?;

        Ok(db_coupon_to_gql(created))
    }

    async fn update_coupon(&self, ctx: &Context<'_>, input: UpdateCouponInput) -> GqlResult<bool> {
        let st = ctx.data_unchecked::<AppState>();
        require_admin(ctx, &st.pool, &st.jwt_secret).await?;

        // Determine owner patch
        let owner_patch: Option<Option<&str>> = if let Some(owner) = input.owner_id.as_deref() {
            Some(Some(owner))
        } else if input.clear_owner.unwrap_or(false) {
            Some(None)
        } else {
            None
        };

        let ok = db::update_coupon_by_code(
            &st.pool,
            &input.code,
            input.description.as_deref(),
            input.service.as_deref(),
            input.expires_in_days,
            owner_patch,
        ).await?;
        Ok(ok)
    }

    async fn delete_coupon(&self, ctx: &Context<'_>, code: String) -> GqlResult<bool> {
        let st = ctx.data_unchecked::<AppState>();
        require_admin(ctx, &st.pool, &st.jwt_secret).await?;
        Ok(db::delete_coupon_by_code(&st.pool, &code).await?)
    }
}

fn require_user(ctx: &Context<'_>, secret: &str) -> anyhow::Result<String> {
    if let Some(uid) = user_id_from_headers(ctx, secret)? {
        Ok(uid)
    } else {
        anyhow::bail!("Unauthorized: missing bearer token");
    }
}
// ---------- Helpers ----------
fn bearer_token_from_ctx(ctx: &Context<'_>) -> Option<String> {
    ctx.data_opt::<axum::http::HeaderMap>()?
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

fn user_id_from_headers(ctx: &Context<'_>, secret: &str) -> anyhow::Result<Option<String>> {
    if let Some(token) = bearer_token_from_ctx(ctx) {
        Ok(Some(auth::parse_jwt(secret, &token)?))
    } else {
        Ok(None)
    }
}

async fn require_admin(ctx: &Context<'_>, pool: &SqlitePool, secret: &str) -> anyhow::Result<()> {
    let Some(user_id) = user_id_from_headers(ctx, secret)? else {
        anyhow::bail!("Unauthorized: missing bearer token");
    };
    if !db::is_user_admin(pool, &user_id).await? {
        anyhow::bail!("Forbidden: admin required");
    }
    Ok(())
}

fn db_coupon_to_gql(c: db::DbCoupon) -> Coupon {
    Coupon {
        id: c.id,
        code: c.code,
        description: c.description,
        service: c.service,
        expires_at: c.expires_at,
        owner_id: c.owner_id,
        created_at: c.created_at,
    }
}
