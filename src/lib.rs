mod dbconn;

pub mod db {
    use serde::{Deserialize, Serialize};
    use sqlx::sqlite::SqlitePool;
    use sqlx::Row;
    use crate::dbconn::dbconnection::connection_pool;

    #[derive(Clone)]
    pub struct Db {
        dbpool: SqlitePool
    }

    #[derive(Deserialize)]
    pub struct CreateUserData {
        pub username: String,
        pub password: String
    }

    #[derive(Deserialize)]
    pub struct CreateRelationship {
        name: String,
        color: Option<String>, // validate
        description: Option<String>,
        user_creator: i32,
        proposed_users: Vec<String>
    }

    impl CreateRelationship {
        pub async fn validate_struct(&self) -> Result<bool, &str> {
            let _ = self.validate_name().await?;
            let _ = self.validate_description().await?;
            let _ = self.validate_color().await?;
            Ok(true)
        }

        async fn validate_description(&self) -> Result<bool, &str> {
            match &self.description {
                None => Ok(true),
                Some(description) => {
                    let num_char = description.chars().count();
                    if num_char > 300 {
                        Err("Description too long")
                    }
                    else {
                        Ok(true)
                    }
                }
            }
        }
        async fn validate_color(&self) -> Result<bool, &str> {
            match &self.color {
                None => Ok(true),
                Some(color_str) => {
                    let first_char = &color_str.chars().next().unwrap();
                    if *first_char != '#' {
                        Err("Color is not an acceptable hex color")
                    } 
                    else {
                        let char_number = &color_str.chars().count();
                        if (*char_number == 4) | (*char_number == 7) {
                            Ok(true)
                        }
                        else {
                            Err("Color is not an acceptable hex color")
                        }
                    }
                }
            }

        }
        async fn validate_name(&self) -> Result<bool, &str> {
            let num_characters = self.name.chars().count();
            if num_characters > 30 {
                Err("Number of characters of name exceed limit")
            }
            else {
                Ok(true)
            }

        }
    }

    #[derive(Serialize)]
    pub struct Relationship {
        pub id: i32,
        pub name: String,
        pub color: String,
        pub description: String,
        pub status: String
    }

    impl Db {
        pub async fn new() -> Db {
            let conn_pool = connection_pool();
            Db {
                dbpool: conn_pool.await
            }
        }

        pub async fn create_relationship(&self, relationship_data:CreateRelationship) -> Result<Relationship, sqlx::Error> {
            let conn = self.dbpool.acquire().await;
            match conn {
                Ok(mut connection) => {
                    // Insert relationship
                    let new_db_row = sqlx::query("insert into relationship (name, color, description) values (?, ?, ?) returning id, name, color, description, status").bind(relationship_data.name).bind(relationship_data.color)
                    .bind(relationship_data.description).fetch_one(&mut *connection).await?;
                    
                    let new_row = Relationship {
                        id: new_db_row.try_get("id")?,
                        name: new_db_row.try_get("name")?,
                        color:new_db_row.try_get("color")?,
                        description:new_db_row.try_get("description")?,
                        status:new_db_row.try_get("status")?
                    };

                    // Insert participants to relationship
                    let new_relationship_user = sqlx::query("insert into relationship_users (user_id, relationship_id, confirmed) values (?, ?, 1)")
                    .bind(relationship_data.user_creator).bind(new_row.id).execute(&mut *connection).await?;

                    if new_relationship_user.rows_affected() != 1 {
                        return Err(sqlx::Error::RowNotFound);
                    }
                    else {
                        for e in relationship_data.proposed_users {
                            let inserted_query = sqlx::query("insert into relationship_user (user_id, relationship_id, confirmed) values ((select users.id from users where users.username = ?), ?, 0)")
                            .bind(e).bind(new_row.id).execute(&mut *connection).await?;
                            if inserted_query.rows_affected() != 1 {
                                return Err(sqlx::Error::RowNotFound);
                            }
                        }
                    }
                    let _ = connection.close().await;
                    Ok(new_row)
                }
                Err(e) => Err(e)
            }
        }

        pub async fn create_user(&self, user_data:CreateUserData) -> Result<u32, sqlx::Error> {
            let conn = self.dbpool.acquire().await;
            match conn {
                Ok(mut connection) => {
                    sqlx::query("insert into users (username, user_password) values (?, ?)").bind(user_data.username).bind(user_data.password).execute(&mut *connection).await.unwrap();
                    let _ = connection.close().await;
                    let id:u32 = 3;
                    
                    Ok(id)
                },
                Err(e) => Err(e)
            }
        }

        pub async fn auth_user(&self, username: String) -> Result<String, sqlx::Error> {
            let conn = self.dbpool.acquire().await;
            match conn {
                Ok(mut connection) => {
                    let user_result = sqlx::query("select users.user_password as password from users where users.username = ?")
                    .bind(username).fetch_one(&mut *connection).await?;
                    let user_password = user_result.try_get("password")?;
                    let _ = connection.close().await;
                    Ok(user_password)
                },
                Err(e) => Err(e)
            }
        }




    }
}