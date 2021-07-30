use std::io::Read;
use std::sync::Arc;
use std::{env, fs};

use async_mutex::Mutex;
use cached::stores::TimedCache;
use slog::o;
use slog::Drain;
use sqlx::postgres::PgPoolOptions;

mod crypto;
mod logging;
mod models;
mod service;
mod utils;

pub type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
struct StringError(String);
impl std::fmt::Display for StringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.0)
    }
}
impl std::error::Error for StringError {}

pub type Result<T> = std::result::Result<T, Error>;

fn env_or(k: &str, default: &str) -> String {
    env::var(k).unwrap_or_else(|_| default.to_string())
}

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::load();

    // The "base" logger that all crates should branch off of
    pub static ref BASE_LOG: slog::Logger = {
        let level: slog::Level = CONFIG.log_level
                .parse()
                .expect("invalid log_level");
        if CONFIG.log_format == "pretty" {
            let decorator = slog_term::TermDecorator::new().build();
            let drain = slog_term::CompactFormat::new(decorator).build().fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            slog::Logger::root(drain, o!())
        } else {
            let drain = slog_json::Json::default(std::io::stderr()).fuse();
            let drain = slog_async::Async::new(drain).build().fuse();
            let drain = slog::LevelFilter::new(drain, level).fuse();
            slog::Logger::root(drain, o!())
        }
    };

    // Base logger
    pub static ref LOG: slog::Logger = BASE_LOG.new(slog::o!("app" => "nihondrill"));

    // state cache
    pub static ref ONE_TIME_TOKENS: Arc<Mutex<TimedCache<String, ()>>> = Arc::new(Mutex::new(TimedCache::with_lifespan(30)));
}

// build a string error
#[macro_export]
macro_rules! se {
    ($($arg:tt)*) => {{ crate::StringError(format!($($arg)*))}};
}

#[macro_export]
macro_rules! resp {
    (json => $obj:expr) => {{
        tide::Response::builder(200)
            .content_type("application/json")
            .body(serde_json::to_string(&$obj)?)
            .build()
    }};
    (status => $status:expr) => {{
        tide::Response::builder($status)
            .content_type("text/plain")
            .build()
    }};
    (status => $status:expr, message => $msg:expr) => {{
        tide::Response::builder($status)
            .content_type("text/plain")
            .body($msg)
            .build()
    }};
}

#[derive(serde::Deserialize)]
pub struct Config {
    pub version: String,

    // host to listen on, defaults to localhost
    pub host: String,
    pub port: u16,

    // json or pretty
    pub log_format: String,
    pub log_level: String,

    // db config
    pub db_url: String,
    pub db_max_connections: u32,

    // key used for generating auth tokens
    pub signing_key: String,
    pub auth_expiration_seconds: u32,
    pub encryption_key: String,
}
impl Config {
    pub fn load() -> Self {
        let version = fs::File::open("commit_hash.txt")
            .map(|mut f| {
                let mut s = String::new();
                f.read_to_string(&mut s).expect("Error reading commit_hash");
                s
            })
            .unwrap_or_else(|_| "unknown".to_string());
        Self {
            version,
            host: env_or("HOST", "localhost"),
            port: env_or("PORT", "3030").parse().expect("invalid port"),
            log_format: env_or("LOG_FORMAT", "json")
                .to_lowercase()
                .trim()
                .to_string(),
            log_level: env_or("LOG_LEVEL", "INFO"),
            db_url: env_or("DATABASE_URL", "error"),
            db_max_connections: env_or("DATABASE_MAX_CONNECTIONS", "5")
                .parse()
                .expect("invalid DATABASE_MAX_CONNECTIONS"),
            // 60 * 60 * 24 * 30
            auth_expiration_seconds: env_or("AUTH_EXPIRATION_SECONDS", "2592000")
                .parse()
                .expect("invalid auth_expiration_seconds"),
            signing_key: env_or("SIGNING_KEY", "01234567890123456789012345678901"),
            encryption_key: env_or("ENCRYPTION_KEY", "01234567890123456789012345678901"),
        }
    }
    pub fn initialize(&self) {
        slog::info!(
            LOG, "initialized config";
            "version" => &CONFIG.version,
            "host" => &CONFIG.host,
            "port" => &CONFIG.port,
            "db_max_connections" => &CONFIG.db_max_connections,
            "log_format" => &CONFIG.log_format,
            "log_level" => &CONFIG.log_level,
            "auth_expiration_seconds" => &CONFIG.auth_expiration_seconds,
        );
    }
    pub fn host(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

async fn new_user<T: AsRef<str> + std::fmt::Debug>(pool: sqlx::PgPool, args: &[T]) {
    if args.len() != 3 {
        eprintln!("invalid args: {:?}", args);
        eprintln!("expected: new $name $email");
    }
    let name = args[1].as_ref().to_string();
    let email = args[2].as_ref().to_string();
    println!("args: {:?}", args);

    let auth_token = service::make_new_auth_token().expect("error generating new auth token");
    let hash = crypto::hmac_sign(&auth_token);

    let u = sqlx::query_as!(
        models::User,
        "
        insert into nd.users
            (name, email) values ($1, $2)
        on conflict (email) do update
            set modified = now(),
                name = $1
        returning *
        ",
        &name,
        &email,
    )
    .fetch_one(&pool)
    .await
    .expect("error inserting new user");

    let expires = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::seconds(
            CONFIG.auth_expiration_seconds as i64,
        ))
        .expect("error creating expiration timestamp");
    let _token = sqlx::query_as!(
        models::AuthToken,
        "
        insert into nd.auth_tokens
            (signature, user_id, expires) values ($1, $2, $3)
        returning *
        ",
        &hash,
        &u.id,
        &expires,
    )
    .fetch_one(&pool)
    .await
    .expect("error inserting new auth token");

    println!("****");
    println!("New token: {}", auth_token);
    println!("****");
}

#[async_std::main]
async fn main() -> Result<()> {
    // try sourcing a .env and server/.env if either exist
    dotenv::dotenv().ok();
    dotenv::from_path(std::env::current_dir().map(|p| p.join(".env")).unwrap()).ok();
    CONFIG.initialize();

    let pool = PgPoolOptions::new()
        .max_connections(CONFIG.db_max_connections)
        .connect(&CONFIG.db_url)
        .await?;

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() > 1 {
        new_user(pool.clone(), &args[1..]).await;
        return Ok(());
    }

    service::start(pool.clone()).await?;
    Ok(())
}
