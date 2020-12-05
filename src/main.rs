use actix_web::{get, http::header, post, web, App, HttpResponse, HttpServer, ResponseError};
use askama::Template;
use concatsql::prelude::*;
use concatsql::html_special_chars;
use serde::Deserialize;
use thiserror::Error;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct AddParams {
    text: String,
}

#[derive(Deserialize)]
struct DeleteParams {
    id: u32,
}

#[derive(Debug)]
struct TodoEntry {
    id:   u32,
    text: String,
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    entries: Vec<TodoEntry>,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),

    #[error("Failed SQL execution")]
    ConcatSQLError(#[from] concatsql::Error),
}

impl ResponseError for MyError {}

#[post("/add")]
async fn add_todo(
    params: web::Form<AddParams>,
    conn: web::Data<Arc<Mutex<Connection>>>,
) -> Result<HttpResponse, MyError> {
    let sql = prep!("INSERT INTO todo (text) VALUES (") + html_special_chars(&params.text).replace('\n', "<br />") + prep!(")");
    conn.lock().unwrap().execute(&sql)?;
    Ok(HttpResponse::SeeOther().header(header::LOCATION, "/").finish())
}

#[post("/delete")]
async fn delete_todo(
    params: web::Form<DeleteParams>,
    conn: web::Data<Arc<Mutex<Connection>>>,
) -> Result<HttpResponse, MyError> {
    let sql = prep!("DELETE FROM todo WHERE id=") + params.id;
    conn.lock().unwrap().execute(&sql)?;
    Ok(HttpResponse::SeeOther().header(header::LOCATION, "/").finish())
}

#[get("/")]
async fn index(conn: web::Data<Arc<Mutex<Connection>>>) -> Result<HttpResponse, MyError> {
    let entries = conn.lock().unwrap().rows("SELECT id, text FROM todo ORDER BY id DESC")?.iter().map(|row| {
        TodoEntry {
            id:   row.get_into(0).unwrap(),
            text: row.get_into(1).unwrap(),
        }
    }).collect::<Vec<_>>();
    let html = IndexTemplate { entries };
    let response_body = html.render()?;
    Ok(HttpResponse::Ok().content_type("text/html").body(response_body))
}

#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
    let conn = Arc::new(Mutex::new(sqlite::open("todo.db").expect("Failed to get the connection.")));
    conn.lock().unwrap().execute(
        "CREATE TABLE IF NOT EXISTS todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL
        )"
    )
    .expect("Failed to create a table `todo`.");

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(add_todo)
            .service(delete_todo)
            .data(conn.clone())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}

