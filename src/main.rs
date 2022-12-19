#[macro_use] extern crate rocket;

use rocket::futures::lock::Mutex;
use rocket::{Rocket, Build, State};
use rocket::serde::{Deserialize, json::Json};
use std::collections::HashMap;
#[derive(Deserialize, Clone)]
struct Project {
    id: String
}

struct GlobalState{
    project_store: Mutex<HashMap<String, Project>>
}

#[post("/", data = "<project>")]
async fn backup_project(project: Json<Project>, state: & State<GlobalState>) -> String {
   
        let map = &mut *state.project_store.lock().await;
        map.insert(project.id.clone(), project.0.clone());
        project.id.clone() 
}

#[get("/<project_id>")]
async fn restore_project(project_id: &str, state: &State<GlobalState>) -> String {
    let map = state.project_store.lock().await;
    map.get(project_id).unwrap().id.clone()
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/project", routes![backup_project, restore_project])
        .manage(GlobalState{project_store: Mutex::from(HashMap::new())})
}