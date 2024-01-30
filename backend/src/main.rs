use std::vec;
use actix_cors::Cors;
use actix_web::{ get, http, web::{self, Data}, App, HttpResponse, HttpServer, Responder };
use serde::{ Serialize, Deserialize };
use std::sync::{Mutex, Arc};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct User {
    username: String,
    email: String,
    age: u32,
}

struct AppState{
    // this is important for making the todos as immutable reference arcs
    pub users: Arc<Mutex<Vec<User>>>,
}

impl AppState {
    // Method to initialize the state with random users
    pub fn init_with_random_users()-> AppState{
        let random_users = fetch_random_users();
        AppState{
            users: Arc::new(Mutex::new(random_users)),
        }
    }
}
// basic cors route
#[get("/hi")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let mut  dereferenced_users = data.users.lock().unwrap();  // This is a clone of the Mutex, not the Vec<User>
    let mut local_users: Vec<User> = Vec::new();
    for item in dereferenced_users.iter_mut(){ // here item is a mutable reference of a user
        item.age = 45;
        local_users.push(item.clone());
    }
    // passing the dereferenced mutex users from App State as Vec
    HttpResponse::Ok().json(local_users)
}

fn fetch_random_users()-> Vec<User>{
    let counter:i32 = 10;
    let mut users: Vec<User> = Vec::new();
    for _ in 0..counter{
        let single_user: User = User{
            username: String::from("rumon"),
            email: "rumon@gmail.com".to_string(),
            age: 27
        };
        users.push(single_user);
    }
    users
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // creating an instance of appState
    let app_state = AppState::init_with_random_users(); 
    let app_data = web::Data::new(app_state);

    let server_result = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b".rust-lang.org")
            })
            .allowed_methods(vec!["GET", "POST", "PATCH"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new().app_data(app_data.clone())
        .wrap(cors)
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
