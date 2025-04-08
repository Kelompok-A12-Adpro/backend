#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, everynyan!"
}

#[get("/<name>")]
fn name(name: & str) -> String {
    format!("Hello, {}!", name)
}

#[catch(404)]
fn not_found(req: &rocket::Request<'_>) -> String {
    format!("Sorry, '{}' is not a valid route..", req.uri())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, name])
        .register("/", catchers![not_found])
}