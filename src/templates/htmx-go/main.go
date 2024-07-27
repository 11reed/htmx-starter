package main

import (
    "database/sql"
    "fmt"
    "log"
    "net/http"
    "os"
    "sync"
    "time"

    "github.com/flosch/pongo2/v6"
    "github.com/gorilla/mux"
    "github.com/joho/godotenv"
    _ "github.com/tursodatabase/libsql-client-go/libsql"
)

type Post struct {
    ID      int
    Title   string
    Content string
}

type CreatePostForm struct {
    Title   string
    Content string
}

var (
    db   *sql.DB
    tpl  *pongo2.Template
	lock sync.Mutex
)

func home(w http.ResponseWriter, r *http.Request) {
    rows, err := db.Query("SELECT id, title, content FROM posts")
    if err != nil {
        http.Error(w, "Failed to execute query", http.StatusInternalServerError)
        return
    }
    defer rows.Close()

    var posts []Post
    for rows.Next() {
        var post Post
        err := rows.Scan(&post.ID, &post.Title, &post.Content)
        if err != nil {
            http.Error(w, "Failed to get row", http.StatusInternalServerError)
            return
        }
        posts = append(posts, post)
    }

    context := pongo2.Context{"posts": posts}
    err = tpl.ExecuteWriter(context, w)
    if err != nil {
        http.Error(w, "Template rendering error", http.StatusInternalServerError)
    }
}

func createPost(w http.ResponseWriter, r *http.Request) {
    var input CreatePostForm
    err := r.ParseForm()
    if err != nil {
        http.Error(w, "Failed to parse form", http.StatusInternalServerError)
        return
    }

    input.Title = r.FormValue("title")
    input.Content = r.FormValue("content")

    _, err = db.Exec("INSERT INTO posts (title, content) VALUES (?, ?)", input.Title, input.Content)
    if err != nil {
        http.Error(w, "Failed to insert post", http.StatusInternalServerError)
        return
    }

    home(w, r)
}

func createTableIfNotExists() {
    _, err := db.Exec(`CREATE TABLE IF NOT EXISTS posts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        content TEXT
    );`)
    if err != nil {
        log.Fatalf("Failed to create table: %v", err)
    }
}

func keepAlive() {
    ticker := time.NewTicker(5 * time.Minute)
    for {
        <-ticker.C
        rows, err := db.Query("SELECT 1")
        if err != nil {
            log.Printf("Failed to keep connection alive: %v", err)
            continue
        }
        rows.Close()
    }
}

func main() {
    err := godotenv.Load()
    if err != nil {
        log.Fatal("Error loading .env file")
    }

    databaseURL := os.Getenv("LIBSQL_URL")
    authToken := os.Getenv("LIBSQL_AUTH_TOKEN")

    if databaseURL == "" {
        log.Fatal("LIBSQL_URL is not set")
    }
    if authToken == "" {
        log.Fatal("LIBSQL_AUTH_TOKEN is not set")
    }

    fmt.Printf("Connecting to database: %s\n", databaseURL)
    
    db, err = sql.Open("libsql", fmt.Sprintf("%s?auth_token=%s", databaseURL, authToken))
    if err != nil {
        log.Fatalf("Failed to connect to database: %v", err)
    }

    tpl, err = pongo2.FromFile("templates/index.html")
    if err != nil {
        log.Fatalf("Failed to load template: %v", err)
    }

    createTableIfNotExists()

    r := mux.NewRouter()
    r.HandleFunc("/", home).Methods("GET")
    r.HandleFunc("/create_post", createPost).Methods("POST")
    r.PathPrefix("/static/").Handler(http.StripPrefix("/static/", http.FileServer(http.Dir("static/"))))

    srv := &http.Server{
        Handler:      r,
        Addr:         "127.0.0.1:3000",
        WriteTimeout: 15 * time.Second,
        ReadTimeout:  15 * time.Second,
    }

    go keepAlive()

    fmt.Println("Listening on 127.0.0.1:3000")
    log.Fatal(srv.ListenAndServe())
}