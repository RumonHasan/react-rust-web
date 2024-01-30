use std::vec;
use actix_cors::Cors;
use actix_web::{
    get,
    http::{ self, StatusCode },
    post,
    web::{ self },
    App,
    HttpResponse,
    HttpServer,
    Responder,
};
use serde::{ Serialize, Deserialize };
use std::sync::{ Mutex, Arc };

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
// main struct
struct User {
    username: String,
    email: String,
    age: u32,
}
// mutable shared app state that is responsible for passing the data
struct AppState {
    // this is important for making the todos as immutable reference arcs
    pub users: Arc<Mutex<Vec<User>>>,
}
// implements of app state
impl AppState {
    pub fn return_curr_app_state()-> AppState{
        AppState{
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
    // Method to initialize the state with random users
    pub fn init_with_random_users() -> AppState {
        let random_users = fetch_random_users();
        AppState {
            users: Arc::new(Mutex::new(random_users)),
        }
    }
    pub fn set_new_user(&self, new_user: User) {
        // locking the users and checking whether they exist or not then only pushing it to the existing users 
        let mut existing_users = self.users.lock().unwrap();
        existing_users.push(new_user);
    }
}
// basic cors route
#[get("/hi")]
async fn index(data: web::Data<AppState>) -> impl Responder {
    let dereferenced_users = data.users.lock().unwrap(); // This is a clone of the Mutex, not the Vec<User>
    let local_users: Vec<User> = dereferenced_users.to_vec();
    HttpResponse::Ok().json(local_users)
}

#[post("/post")]
async fn post_users(body: web::Json<User>, data: web::Data<AppState>) -> impl Responder {
    let new_user: User = body.into_inner(); // into inner is used to consumer the JSON body
    data.get_ref().set_new_user(new_user.clone()); // passes the new set of users
    HttpResponse::build(StatusCode::OK).body("You managed to post it... jeezus boi!")
}

fn fetch_random_users() -> Vec<User> {
    let counter: i32 = 10;
    let mut users: Vec<User> = Vec::new();
    for _ in 0..counter {
        let single_user: User = User {
            username: String::from("rumon"),
            email: "rumon@gmail.com".to_string(),
            age: 27,
        };
        users.push(single_user);
    }
    users
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // creating an instance of appState
    let app_state = AppState::return_curr_app_state(); // initial renders
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

        App::new().app_data(app_data.clone()).wrap(cors).service(index).service(post_users)
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
