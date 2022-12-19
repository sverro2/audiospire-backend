#[macro_use] extern crate rocket;

use rocket::{Rocket, Build};
use rocket::serde::{Deserialize, json::Json};

#[derive(Deserialize)]
struct Project<'r> {
    id: &'r str
}

#[post("/", data = "<project>")]
fn backup_project(project: Json<Project>) -> &str {
    project.id
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/project", routes![backup_project])
}