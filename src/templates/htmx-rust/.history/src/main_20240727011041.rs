use axum::{
    extract::{Extension, Form},
    response::Html,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use libsql::{Builder, Connection, Database, params};
use serde::{Deserialize, Serialize};
use tera::Tera;
use std::env;
use std::sync::Arc;
use tokio::time::{self, Duration};

#[derive(Serialize)]
struct Post {
    id: i32,
    title: String,
    content: Option<String>,
}

#[derive(Deserialize)]
struct CreatePostForm {
    title: String,
    content: Option<String>,
}

async fn home(
    Extension(tera): Extension<Tera>,
    Extension(db): Extension<Arc<Database>>,
) -> Result<Html<String>, axum::http::StatusCode> {
    // let conn = db.connect().unwrap();
    let conn = match db.connect() {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Database connection error: {:?}", err);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // let mut stmt = conn.prepare("SELECT id, title, content FROM posts").await.unwrap();
    let mut stmt = match conn.prepare("SELECT id, title, content FROM posts").await {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Failed to prepare statement: {:?}", err);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // let mut rows = stmt.query(()).await.unwrap();
    let mut rows = match stmt.query(()).await {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Failed to execute query: {:?}", err);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut posts = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        posts.push(Post {
            id: row.get(0).unwrap(),
            title: row.get(1).unwrap(),
            content: row.get(2).unwrap(),
        });
    }

    let mut context = tera::Context::new();
    context.insert("posts", &posts);

    tera.render("index.html", &context)
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_post(
    Extension(db): Extension<Arc<Database>>,
    Extension(tera): Extension<Tera>,
    Form(input): Form<CreatePostForm>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let conn = db.connect().unwrap();
    let content_value = input.content.as_deref().unwrap_or("");

    conn.execute(
        "INSERT INTO posts (title, content) VALUES (?, ?)",
        params![input.title, content_value],
    )
    .await
    .unwrap();

    let mut stmt = conn.prepare("SELECT id, title, content FROM posts").await.unwrap();
    let mut rows = stmt.query(()).await.unwrap();
    
    let mut posts = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        posts.push(Post {
            id: row.get(0).unwrap(),
            title: row.get(1).unwrap(),
            content: row.get(2).unwrap(),
        });
    }

    let mut context = tera::Context::new();
    context.insert("posts", &posts);

    tera.render("index.html", &context)
        .map(Html)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_table_if_not_exists(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT
        );",
        ()
    ).await.unwrap();
}

async fn keep_alive(db: Arc<Database>) {
    let mut interval = time::interval(Duration::from_secs(300)); // Ping every 5 minutes
    loop {
        interval.tick().await;
        let conn = db.connect().unwrap();
        if let Err(err) = conn.execute("SELECT 1", ()).await {
            eprintln!("Failed to keep connection alive: {:?}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("RUST_LOG: {:?}", std::env::var("RUST_LOG").ok());

    tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

    let database_url = env::var("LIBSQL_URL").expect("LIBSQL_URL must be set");
    let auth_token = env::var("LIBSQL_AUTH_TOKEN").unwrap_or_default();

    let db = Arc::new(
        Builder::new_remote(database_url, auth_token)
            .build()
            .await
            .expect("Failed to build database"),
    );

    create_table_if_not_exists(&db.connect().unwrap()).await;

    let tera = Tera::new("templates/**/*").unwrap();
    let serve_dir = ServeDir::new("static");

    tokio::spawn(keep_alive(db.clone()));

    let app = Router::new()
        .route("/", get(home))
        // .route("/index.html", get(home))
        .route("/create_post", post(create_post))
        .layer(Extension(tera))
        .layer(Extension(db))
        .nest_service("/static", serve_dir);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
