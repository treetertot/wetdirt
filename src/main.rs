use reqwest::Client;
use wetdirt::actor::UserManager;
use wetdirt::db::Database;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let client = Client::new();
    let db = Database::new(
        client,
        "http://localhost:8000/sql",
        "test",
        "test",
        "root:root",
    )
    .await
    .unwrap();

    let manager = UserManager::new(db);

    let q = manager.create_user("soupconsort", "smellybean").await;
    println!("{:?}", q);

    //let app = Router::new()
    //    .route("/", get(|| async { "Hello, World!" }))
    //    .route("/about", {
    //        let rendered = "soup".to_string();
    //        get(|| async move { Html(rendered.clone()) })
    //    });
    //
    //// run it with hyper on localhost:3000
    //axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    //    .serve(app.into_make_service())
    //    .await
    //    .unwrap();
}
