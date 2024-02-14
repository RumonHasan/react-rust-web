mod model;
mod handler;
mod schema;

use std::{ vec };
use actix_cors::Cors;
use actix_web::{
    delete,
    get,
    http::{ self, StatusCode },
    patch,
    post,
    web::{ self },
    App,
    HttpResponse,
    HttpServer,
    Responder,
};
use model::{NoteModel, NoteModelResponse};
use serde::{ Deserialize, Serialize };
use uuid::Uuid;
use std::sync::{ Mutex, Arc };
use rand::{ self, Rng };
use sqlx::mysql::{ MySqlPool as CustomMySqlPool, MySqlPoolOptions as CustomMySqlPoolOptions };

use crate::schema::{CreateNoteSchema, FilterOptions};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
// main struct
struct User {
    id: Uuid,
    username: String,
    email: String,
    age: u32,
    comments: Vec<Comment>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Comment {
    comment_id: Uuid,
    comment: String,
}
// mutable shared app state that is responsible for passing the data
struct AppState {
    // this is important for making the todos as immutable reference arcs
    pub users: Arc<Mutex<Vec<User>>>,
}

struct DatabaseState {
    pub db: CustomMySqlPool, // custom mysql database
}

// implements of app state and its sub functions
impl AppState {
    // pushing the new user to the existing app state
    pub fn set_new_user(&self, new_user: User) {
        let mut existing_users = self.users.lock().unwrap();
        existing_users.push(new_user);
    }
    pub fn generate_uuid() -> Uuid {
        let new_uuid = Uuid::new_v4();
        new_uuid
    }
    pub fn generate_random_users(user_count: i32) -> AppState {
        let mut random_users: Vec<User> = Vec::new();
        for _ in 0..user_count {
            let new_user: User = User {
                id: AppState::generate_uuid(),
                username: String::from("something"),
                email: String::from("something"),
                age: 27,
                comments: AppState::generate_some_comments(5),
            };
            random_users.push(new_user);
        }
        AppState {
            users: Arc::new(Mutex::new(random_users)),
        }
    }
    // generating some random comments for each user
    pub fn generate_some_comments(comment_count: i32) -> Vec<Comment> {
        let mut comments: Vec<Comment> = Vec::new();
        let comment_text_vec: Vec<String> = vec![
            String::from("Hello and hi"),
            String::from(" hi"),
            String::from("Hello hi"),
            String::from("hi")
        ];
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..comment_text_vec.len());
        for _ in 0..comment_count {
            comments.push(Comment {
                comment_id: AppState::generate_uuid(),
                comment: comment_text_vec[random_index].to_string(),
            });
        }
        comments
    }
}

// posting to sql database

// basic cors route
#[get("/hi")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let dereferenced_users = data.users.lock().unwrap(); // This is a clone of the Mutex, not the Vec<User>
    let local_users: Vec<User> = dereferenced_users.to_vec();
    HttpResponse::Ok().json(local_users)
}

// for posting a single todo
#[post("/post")]
async fn post_users(body: web::Json<User>, data: web::Data<AppState>) -> impl Responder {
    let new_user: User = body.into_inner(); // into inner is used to consumer the JSON body
    data.get_ref().set_new_user(new_user.clone()); // passes the new set of users
    HttpResponse::build(StatusCode::OK).body("You managed to post it... jeezus boi!")
}

// posting notes on note schema
#[get("/post-notes")]
pub async fn get_note(
    opts: web::Query<FilterOptions>,
    data: web::Data<DatabaseState>,
) -> impl Responder {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let notes: Vec<NoteModel> = sqlx::query_as!(
        NoteModel,
        r#"SELECT * FROM notes ORDER by id LIMIT ? OFFSET ?"#,
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await
    .unwrap();

    let note_responses = notes
        .into_iter()
        .map(|note| filter_db_record(&note))
        .collect::<Vec<NoteModelResponse>>();

    let json_response = serde_json::json!({
        "status": "success",
        "results": note_responses.len(),
        "notes": note_responses
    });
    HttpResponse::Ok().json(json_response)
}

fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        title: note.title.to_owned(),
        content: note.content.to_owned(),
        category: note.category.to_owned().unwrap(),
        published: note.published != 0,
        createdAt: note.created_at.unwrap(),
        updatedAt: note.updated_at.unwrap(),
    }
}

// for receiving a new comment and adding it to the AppState
#[post("/comment-post/{id}")]
async fn post_comment(
    user_id: web::Path<String>,
    body: web::Json<Comment>,
    data: web::Data<AppState>
) -> impl Responder {
    let user_id = user_id.to_string();
    let new_comment: Comment = body.into_inner();
    match
        data.users
            .lock()
            .unwrap()
            .iter_mut()
            .find(|user| user.id.to_string() == user_id)
    {
        Some(found_user) => {
            found_user.comments.push(new_comment);
        }
        None => {
            HttpResponse::NotFound();
        }
    }
    HttpResponse::Ok()
}
// deleting a comment
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct DeleteComment {
    comment_id: String,
    user_id: String,
}
#[post("/comment-delete")]
async fn delete_comment(
    body: web::Json<DeleteComment>,
    data: web::Data<AppState>
) -> impl Responder {
    let delete_body = body.into_inner();
    let user_id = &delete_body.user_id;
    let comment_id = &delete_body.comment_id;
    // delete comment logic
    match
        data.users
            .lock()
            .unwrap()
            .iter_mut()
            .find(|user| user.id.to_string() == user_id.to_string())
    {
        Some(found_user) => {
            // directly modifies the element without explicitly releasing the mutex guard lock
            found_user.comments.retain(
                |comment| comment.comment_id.to_string() != comment_id.to_string()
            );
        }
        None => {
            HttpResponse::NotFound();
        }
    }
    HttpResponse::Ok()
}

// update route
#[patch("/update/{id}")]
async fn update_user(
    id: web::Path<String>,
    body: web::Json<User>,
    data: web::Data<AppState>
) -> impl Responder {
    let mut existing_users = data.users.lock().unwrap();
    let update_id = id.to_string();
    // updating the user through an iterable reference
    if
        let Some(existing_user) = existing_users
            .iter_mut()
            .find(|user| user.id.to_string() == update_id)
    {
        let new_user_body = body.into_inner();
        existing_user.age = new_user_body.age;
        existing_user.username = new_user_body.username;
        existing_user.email = new_user_body.email;
    }
    HttpResponse::Ok()
}

// deleting a user
#[delete("/delete/{id}")]
async fn delete_user(id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let mut mutable_user_ref = data.users.lock().unwrap();
    let delete_id = id.to_string();
    let delete_user = mutable_user_ref.iter().find(|val| val.id.to_string() == delete_id);
    if delete_user.is_none() {
        return HttpResponse::NoContent().finish();
    }
    mutable_user_ref.retain(|user| user.id.to_string() != delete_id);
    HttpResponse::build(StatusCode::OK).body("User has been deleted successfully")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // creating an instance of appState
    let app_state = AppState::generate_random_users(10); // initial renders
    let app_data = web::Data::new(app_state);

    // sqlx data base connect
    let database_url = "mysql://admin:password123@localhost:6500/rust_sqlx"; // default url setup
    let pool = match CustomMySqlPoolOptions::new().max_connections(10).connect(&database_url).await {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    // local host server code
    let server_result = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b".rust-lang.org")
            })
            // the methods need to be allowed here in order for axios in the frontend to access it
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(DatabaseState { db: pool.clone() }))
            .wrap(cors)
            .service(get_note)
            .service(index)
    })
        .bind(("127.0.0.1", 8080))?
        .run().await;

    // if server is found then show ok or error
    match server_result {
        Ok(_) => {
            println!("Server is running on http://127.0.0.1:8080");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

//general notes
//alternative way filtering and cloning the owned instanced of the comment trait
// let new_comments: Vec<Comment> = found_user.comments
//     .iter()
//     .filter(|comment| comment.comment_id.to_string() != comment_id.to_string())
//     .map(|comment| comment.clone())
//     .collect();
// found_user.comments = new_comments;
