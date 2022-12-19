#[macro_use] extern crate rocket;

use rocket::{Rocket, Build};
use rocket::serde::{Deserialize, json::Json};
use std::collections::HashMap;

#[derive(Deserialize)]
struct Project<'a> {
    id: &'a str
}

struct GlobalState<'a>{
    projectStore: HashMap<&'a str, Project<'a>>
}

#[post("/", data = "<project>")]
fn backup_project(project: Json<Project>) -> &str {
    project.id
}

#[get("/<project_name>")]
fn restore_project(project_name: &str) -> &str {

    project_name
}



// #[put("/media", data = "<project>")]
// fn backup_project(project: Json<Project>) -> &str {
//     project.id
// }

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/project", routes![backup_project, restore_project])
        .manage(GlobalState{projectStore: HashMap::new()})
}