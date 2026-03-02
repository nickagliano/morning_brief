use axum::{routing::get, Json, Router};
use chrono::{Duration, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Task {
    text: String,
    done: bool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let todo_url  = std::env::var("SIMPLE_TODO_URL").unwrap_or_else(|_| "http://localhost:8765".into());
    let txtme_url = std::env::var("TXTME_URL").unwrap_or_else(|_| "http://localhost:5543".into());
    let txtme_key = std::env::var("TXTME_API_KEY").unwrap_or_default();
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(5544);

    tokio::spawn(scheduler_loop(todo_url, txtme_url, txtme_key));

    let app = Router::new().route("/health", get(health));
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await.unwrap();
    println!("[morning_brief] listening on :{port}");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn scheduler_loop(todo_url: String, txtme_url: String, txtme_key: String) {
    loop {
        let wait = secs_until_9am_eastern();
        println!("[morning_brief] next brief in {}m", wait.as_secs() / 60);
        tokio::time::sleep(wait).await;

        match send_brief(&todo_url, &txtme_url, &txtme_key).await {
            Ok(())  => println!("[morning_brief] sent"),
            Err(e) => eprintln!("[morning_brief] error: {e}"),
        }

        // Guard against firing twice in the same minute
        tokio::time::sleep(std::time::Duration::from_secs(90)).await;
    }
}

fn secs_until_9am_eastern() -> std::time::Duration {
    let now_et  = Utc::now().with_timezone(&New_York);
    let nine_am = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

    let target_date = if now_et.time() < nine_am {
        now_et.date_naive()
    } else {
        now_et.date_naive() + Duration::days(1)
    };

    let target_et  = New_York.from_local_datetime(&target_date.and_time(nine_am)).unwrap();
    let secs       = (target_et.with_timezone(&Utc) - Utc::now()).num_seconds().max(0) as u64;
    std::time::Duration::from_secs(secs)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

async fn send_brief(
    todo_url:  &str,
    txtme_url: &str,
    txtme_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let tasks: Vec<Task> = client
        .get(format!("{todo_url}/tasks"))
        .send()
        .await?
        .json()
        .await?;

    let house: Vec<Task> = client
        .get(format!("{todo_url}/house-projects"))
        .send()
        .await?
        .json()
        .await?;

    let top_task  = tasks.iter().find(|t| !t.done);
    let top_house = house.iter().find(|t| !t.done);

    // PORT: MESSAGE_FORMAT
    let message = match (top_task, top_house) {
        (None, None) => "GM! All clear today.".to_string(),
        (Some(t), None)  => format!("GM!\nTodo: {}\n(Home: all clear)", truncate(&t.text, 50)),
        (None, Some(h))  => format!("GM!\n(Todo: all clear)\nHome: {}", truncate(&h.text, 50)),
        (Some(t), Some(h)) => format!("GM!\nTodo: {}\nHome: {}", truncate(&t.text, 50), truncate(&h.text, 50)),
    };

    client
        .post(format!("{txtme_url}/notify"))
        .header("X-Api-Key", txtme_key)
        .json(&json!({"message": message}))
        .send()
        .await?;

    Ok(())
}
