#[macro_use]
extern crate rocket;

use rocket::data::{Data, ToByteUnit};
use rocket::fairing::{self, AdHoc};
use rocket::futures::lock::Mutex;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{Build, Rocket, State};
use rocket_db_pools::{sqlx, Connection, Database};
use std::collections::HashMap;
use futures_util::{stream::TryStreamExt, future::TryFutureExt};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::SqlitePool);

#[derive(Deserialize, Serialize, Clone)]
struct Project {
    id: String,
}

struct GlobalState {
    media_store: Mutex<HashMap<String, Vec<u8>>>,
}

#[post("/", data = "<project>")]
async fn backup_project(mut db: Connection<Db>, project: Json<Project>) -> Result<Created<Json<Project>>> {
    // There is no support for `RETURNING`.
    sqlx::query!("INSERT INTO projects_v1 (id) VALUES (?)", project.id)
        .execute(&mut *db)
        .await?;

    Ok(Created::new("/").body(project))
}

#[get("/<project_id>")]
async fn restore_project(project_id: &str, mut db: Connection<Db>) -> Option<Json<Project>> {
    sqlx::query!("SELECT id FROM projects_v1 WHERE id = ?", project_id)
    .fetch_one(&mut *db)
    .map_ok(|r| Json(Project { id: r.id }))
    .await
    .ok()
}

#[post("/<project_id>/<media_id>", data = "<media>")]
async fn backup_media<'a>(
    project_id: &str,
    media_id: &str,
    media: Data<'_>,
    state: &State<GlobalState>,
) -> std::io::Result<&'a str> {
    let map = &mut *state.media_store.lock().await;

    let media_bytes = media.open(50.megabytes()).into_bytes().await?;

    map.insert(String::from(media_id), media_bytes.value);
    Ok("done")
}

#[get("/<project_id>/<media_id>")]
async fn restore_media(
    project_id: &str,
    media_id: &str,
    state: &State<GlobalState>,
) -> Option<Vec<u8>> {
    let media_store = &mut *state.media_store.lock().await;

    media_store.get(media_id).map(Vec::clone)
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/sqlx/migrations").run(&**db).await {
            Ok(_) => {
                print!("Migrations succesfully run");
                Ok(rocket)
            }
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount(
            "/project",
            routes![backup_project, restore_project, backup_media, restore_media],
        )
        .manage(GlobalState {
            media_store: Mutex::from(HashMap::new()),
        })
        .attach(Db::init())
        .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
        .mount("/sqlx", routes![])
    // .mount("/diesel", routes![])
}
