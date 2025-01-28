use axum::{extract::{Query, State}, http::StatusCode, routing::{delete, get, post, put}, Json, Router};
//use axum_macros::debug_handler;
use datesapp::db::{CreateRelationship, EditRelationship, CreateUserData, Db, Relationship, MultipleRelationship};
use bcrypt::{DEFAULT_COST, hash, verify};
use serde::Deserialize;
// use datesapp::
use std::collections::HashMap;


#[derive(Deserialize)]
struct RelationshipQuery {
    user_id: u32,
    relationship_id: u32
}

#[tokio::main]
async fn main() {
    let dconn = Db::new().await;
    let app: Router = Router::new().route("/createuser", post(route_create_user))
                    .route("/auth", post(route_auth_user))
                    .route("/createrelation", post(route_create_relationship))
                    .route("/accept-relationship", get(route_accept_relationship))
                    .route("/edit-relationship", put(route_edit_relationship))
                    .route("/delete-relationship", delete(route_delete_relationship))
                    .route("/get-relationship", get(route_get_relationship))
                    .with_state(dconn);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn route_get_relationship(State(stateconn): State<Db>, user_id: Query<u32>) -> (StatusCode, Json<MultipleRelationship>) {
    let result = stateconn.get_relationships(user_id.0).await.unwrap();
    (StatusCode::OK, Json(result))
}

async fn route_edit_relationship(State(stateconn): State<Db>, Json(payload): Json<EditRelationship>) -> (StatusCode, Json<Relationship>) {
    match payload.validate_struct().await {
        Err(e) => panic!("{}", e),
        Ok(_) => {
            match stateconn.edit_relationship(payload).await {
                Err(e) => panic!("{}", e), //(StatusCode::UNPROCESSABLE_ENTITY, format!("Error: {}", e)),
                Ok(data) => (StatusCode::OK, Json(data))
            }
        }
    }
}

async fn route_accept_relationship(State(stateconn): State<Db>, query_param: Query<RelationshipQuery>) -> (StatusCode, String) {
    match stateconn.accept_relationship(query_param.user_id,query_param.relationship_id).await {
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, format!("Error: {}", e)),
        Ok(_) => (StatusCode::OK, String::from("Relationship accepted succesfully"))
    }
}

async fn route_delete_relationship(State(stateconn): State<Db>, query_param: Query<RelationshipQuery>) -> (StatusCode, String) {
    match stateconn.delete_relationship(query_param.user_id, query_param.relationship_id).await {
        Err(e) => (StatusCode::FORBIDDEN, format!("Error: {}", e)),
        Ok(_) => (StatusCode::OK, String::from("Relationship deleted succesfully"))
    }
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