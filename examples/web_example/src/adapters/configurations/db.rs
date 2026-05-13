use mongodb::options::Credential;
use mongodb::{
    bson::doc,
    options::{ClientOptions, ReadConcernLevel, WriteConcern},
    Client,
};
use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::adapters::configurations::settings::{get_app_settings, DbCredentials};

static MONGO_CLIENT: OnceCell<Arc<Client>> = OnceCell::const_new();

pub async fn init() {
    let settings = get_app_settings();
    let client = get_mongodb_client(settings.database.clone()).await;

    // To check database connection
    let database = client.database("admin");
    let ping_command = doc! { "ping": 1 };
    // Send a ping to confirm a successful connection
    let ping_result = database.run_command(ping_command).await;
    match ping_result {
        Ok(result) => {
            println!("Answer from server: {:?}", result);
            println!("✅ Database connected successfully");
        }
        Err(e) => {
            println!("❎ Database connection error: {}", e);
        }
    }
}

async fn get_mongodb_client(db_settings: DbCredentials) -> Arc<Client> {
    MONGO_CLIENT
        .get_or_init(|| async {
            let mut client_options = ClientOptions::parse(db_settings.url)
                .await
                .expect("Failed to parse options from URI");
            let credential = Credential::builder()
                .username(Some(db_settings.username))
                .password(Some(db_settings.password))
                .source(Some("admin".to_string())) // Database where the user was created
                .build();

            client_options.read_concern = Some(ReadConcernLevel::Majority.into());
            client_options.write_concern = Some(WriteConcern::majority());
            client_options.app_name = Some("ExampleApp".to_string());
            client_options.max_pool_size = Some(16);
            client_options.min_pool_size = Some(1);
            client_options.max_connecting = Some(5);
            client_options.credential = Some(credential);

            Arc::new(Client::with_options(client_options).expect("Failed to create MongoDB Client"))
        })
        .await
        .to_owned()
}

pub async fn get_initialized_mongodb_client() -> Arc<Client> {
    MONGO_CLIENT.get().unwrap().clone()
}
