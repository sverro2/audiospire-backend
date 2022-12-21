#[macro_use] extern crate rocket;

use rocket::futures::lock::Mutex;
use rocket::{Rocket, Build, State};
use rocket::data::{Data, ToByteUnit};
use rocket::serde::{Deserialize, Serialize, json::Json};
use std::collections::HashMap;
use rocket::response::status;

// project opslaan in database
// media opslaan als file
// media openen van file
// authenticatie
// aanspreken vanuit

#[derive(Deserialize, Serialize, Clone)]
struct Project {
    id: String
}

struct GlobalState{
    project_store: Mutex<HashMap<String, Project>>,
    media_store: Mutex<HashMap<String, Vec<u8>>>
}

#[post("/", data = "<project>")]
async fn backup_project(project: Json<Project>, state: &State<GlobalState>) -> status::Accepted<String> {
    let map = &mut *state.project_store.lock().await;
    map.insert(project.id.clone(), project.0.clone());
    status::Accepted(Some(format!("id: '{}'", project.id.clone())))
}

#[get("/<project_id>")]
async fn restore_project(project_id: &str, state: &State<GlobalState>) -> Option<Json<Project>> {
    let project_store = state.project_store.lock().await;

    project_store.get(project_id).map(|project| {
        Json(project.clone())
    })
}

#[post("/<project_id>/<media_id>", data = "<media>")]
async fn backup_media(
    project_id: &str, 
    media_id: &str, 
    media: Data<'_>,
    state: &State<GlobalState>
) -> std::io::Result<String> {
    let map = &mut *state.media_store.lock().await;

    let media_bytes = media.open(50.megabytes()).into_bytes().await?;

    map.insert(String::from(media_id), media_bytes.value);
    Ok(String::from("done"))
}

#[get("/<project_id>/<media_id>")]
async fn restore_media<'a>(
    project_id: &str, 
    media_id: &str, 
    state: &'a State<GlobalState>
) -> Option<Vec<u8>> {
    let media_store = &mut *state.media_store.lock().await;

    media_store.get(media_id).map(Vec::clone)
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/project", routes![backup_project, restore_project, backup_media, restore_media])
        .manage(GlobalState{
            project_store: Mutex::from(HashMap::new()),
            media_store: Mutex::from(HashMap::new())
        })
}