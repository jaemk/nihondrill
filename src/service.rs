use crate::{crypto, models, resp, Result, CONFIG, LOG};

macro_rules! user_or_redirect {
    ($req:expr) => {{
        let user = get_auth_user(&$req).await;
        if user.is_none() {
            return Ok(tide::Redirect::new("/").into());
        }
        user.unwrap()
    }};
}

#[allow(unused_macros)]
macro_rules! params_or_error {
    ($req:expr, $param_type:ty) => {{
        match $req.query::<$param_type>() {
            Err(e) => {
                slog::error!(LOG, "invalid recent query params {:?}", e);
                return Ok(resp!(status => 400, message => "invalid query parameters"));
            }
            Ok(params) => params,
        }
    }};
}

#[derive(Clone)]
struct Context {
    pool: sqlx::PgPool,
}

pub async fn start(pool: sqlx::PgPool) -> crate::Result<()> {
    let ctx = Context { pool };
    let mut app = tide::with_state(ctx);
    app.at("/").all(index);
    app.at("/status").all(status);
    app.with(crate::logging::LogMiddleware::new());

    slog::info!(LOG, "running at {}", crate::CONFIG.host());
    app.listen(crate::CONFIG.host()).await?;
    Ok(())
}

async fn index(req: tide::Request<Context>) -> tide::Result {
    let user = user_or_redirect!(req);
    return Ok(resp!(status => 200, message => format!("Hello, {}!", user.name)));
}

#[derive(serde::Serialize)]
struct Status<'a> {
    ok: &'a str,
    version: &'a str,
}

async fn status(_req: tide::Request<Context>) -> tide::Result {
    Ok(resp!(json => Status {
        ok: "ok",
        version: &CONFIG.version
    }))
}

pub fn make_new_auth_token() -> Result<String> {
    let s = uuid::Uuid::new_v4()
        .to_simple()
        .encode_lower(&mut uuid::Uuid::encode_buffer())
        .to_string();
    let n = crypto::rand_bytes(16)?;
    let s = format!("{}:{}", hex::encode(n), s);
    let b = crate::crypto::hash(s.as_bytes());
    Ok(hex::encode(&b))
}

async fn get_auth_user(req: &tide::Request<Context>) -> Option<models::User> {
    let ctx = req.state();
    match req.cookie("auth_token") {
        None => {
            slog::info!(LOG, "no auth token cookie found");
            None
        }
        Some(cookie) => {
            let token = cookie.value();
            let hash = crypto::hmac_sign(token);
            let u = sqlx::query_as!(
                models::User,
                "
                select u.*
                from nd.auth_tokens t
                    inner join nd.users u
                    on u.id = t.user_id
                where signature = $1
                ",
                &hash,
            )
            .fetch_one(&ctx.pool)
            .await
            .ok();

            slog::debug!(LOG, "current user {:?}", u);
            if let Some(ref u) = u {
                sqlx::query!(
                    "delete from nd.auth_tokens where user_id = $1 and expires <= now()",
                    &u.id
                )
                .execute(&ctx.pool)
                .await
                .map_err(|e| {
                    format!(
                        "error deleting expired auth tokens for user {}, continuing: {:?}",
                        u.id, e
                    )
                })
                .ok();
            }
            u
        }
    }
}
