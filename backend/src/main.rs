use std::vec;
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
use serde::{ Serialize, Deserialize };
use uuid::Uuid;
use std::sync::{ Mutex, Arc };

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
// main struct
struct User {
    id: Uuid,
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
    pub fn return_curr_app_state() -> AppState {
        AppState {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
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
            };
            random_users.push(new_user);
        }
        AppState {
            users: Arc::new(Mutex::new(random_users)),
        }
    }
}
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

    // local host server code
    let server_result = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:5173")
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes().ends_with(b".rust-lang.org")
            })
            // the methods need to be allowed here in order for axios in the frontend to access it
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(app_data.clone())
            .wrap(cors)
            .service(index)
            .service(post_users)
            .service(delete_user)
            .service(update_user)
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
