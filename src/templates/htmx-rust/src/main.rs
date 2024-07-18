use axum::{
    extract::{Extension, Form},
    response::Html,
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use libsql::{Builder, Connection, params};
use serde::{Deserialize, Serialize};
use tera::Tera;
use std::env;

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
    Extension(conn): Extension<Connection>,
) -> Result<Html<String>, axum::http::StatusCode> {
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

async fn create_post(
    Extension(conn): Extension<Connection>,
    Extension(tera): Extension<Tera>,
    Form(input): Form<CreatePostForm>,
) -> Result<Html<String>, axum::http::StatusCode> {
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

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = env::var("LIBSQL_URL").expect("LIBSQL_URL must be set");
    let auth_token = env::var("LIBSQL_AUTH_TOKEN").unwrap_or_default();

    let db = Builder::new_remote(database_url, auth_token)
        .build()
        .await
        .expect("Failed to build database");
    let conn = db.connect().unwrap();

    create_table_if_not_exists(&conn).await;

    let tera = Tera::new("templates/**/*").unwrap();

    let serve_dir = ServeDir::new("static");

    let app = Router::new()
        .route("/", get(home))
        .route("/create_post", post(create_post))
        .layer(Extension(tera))
        .layer(Extension(conn))
        .nest_service("/static", serve_dir);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}