mod dbconn;

pub mod db {
    use serde::Deserialize;
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

    impl Db {
        pub async fn new() -> Db {
            let conn_pool = connection_pool();
            Db {
                dbpool: conn_pool.await
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