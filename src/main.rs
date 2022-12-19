#[macro_use] extern crate rocket;
use rocket::serde::{Deserialize, json::Json};


#[derive(Deserialize)]
struct Project<'r> {
    id: &'r str
}
// Post project

#[post("/", data = "<project>")]
fn backup_project(project: Json<Project>) -> &str {
    project.id
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/project", routes![backup_project])
}
