use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
//use axum_macros::debug_handler;
use datesapp::db::{CreateRelationship, CreateUserData, Db, Relationship };
use bcrypt::{DEFAULT_COST, hash, verify};
// use datesapp::
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let dconn = Db::new().await;
    let app: Router = Router::new().route("/createuser", post(route_create_user))
                    .route("/auth", post(route_auth_user))
                    .route("/createrelation", post(route_create_relationship))
                    .with_state(dconn);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn route_create_relationship(State(stateconn): State<Db>, Json(payload): Json<CreateRelationship>) -> (StatusCode, Json<Relationship>) {
    // Return custom error implementing into respone in the near future
    match payload.validate_struct().await {
        Ok(_) => {
            match stateconn.create_relationship(payload).await {
                Ok(new_row) => (StatusCode::OK, Json(new_row)),
                Err(e) => panic!("{e}")//(StatusCode::UNPROCESSABLE_ENTITY, e)
            }
        },
        Err(e) => panic!("{e}")//(StatusCode::UNPROCESSABLE_ENTITY, e)
    }
}

async fn route_auth_user(State(stateconn): State<Db>, Json(payload): Json<CreateUserData>) -> (StatusCode, String) {
    // Select from user table username
    match stateconn.auth_user(payload.username).await {
        Ok(hashed_pasword) => {
            match verify(payload.password, &hashed_pasword) {
                Ok(auth_result) => {
                    if auth_result == true {
                        (StatusCode::ACCEPTED, String::from("true"))
                    }
                    else {
                        (StatusCode::UNAUTHORIZED, String::from("Password Incorrect"))
                    }
                }
                Err(_) => (StatusCode::UNAUTHORIZED, String::from("Password Incorrect"))
            }
        }
        Err(e) => (StatusCode::UNAUTHORIZED, format!("User does not exist of password incorrect: {}", e))
    }
}

async fn route_create_user(State(stateconn): State<Db>, Json(mut payload): Json<CreateUserData>) -> (StatusCode, String) {
    // encrypt password
    match hash(payload.password,DEFAULT_COST) {
        Ok(password)=> {
            payload.password = password;
        
            let mut response = HashMap::new();
            response.insert("message", "User Created Succesfully");
        
            // call create user function
            let id = stateconn.create_user(payload);
            match id.await {
                Ok(_) => (StatusCode::OK, String::from("User Created Succesfully")),
                Err(err) => (StatusCode::UNPROCESSABLE_ENTITY, format!("Error creating user {}", err))
            }
        },    
        Err(err) => {
            let error_string = format!("Password provided is not acceptable {}", err);
            (StatusCode::UNPROCESSABLE_ENTITY, error_string)
        }    

    }    



}