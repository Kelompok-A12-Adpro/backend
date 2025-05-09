#[macro_use]
extern crate rocket;

use backend::controller::admin::routes::admin_routes;

#[get("/")]
fn index() -> &'static str {
    "Hello, everynyan!"
}

#[catch(404)]
fn not_found(req: &rocket::Request<'_>) -> String {
    format!("Sorry, '{}' is not a valid route..", req.uri())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/admin", admin_routes())
        .register("/", catchers![not_found])
}
