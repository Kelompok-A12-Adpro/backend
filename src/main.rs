#[macro_use]
extern crate rocket;

use backend::controller::admin::routes::admin_routes;
use backend::service::admin::platform_statistics_service::PlatformStatisticsService;

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
    let platform_statistics_service = PlatformStatisticsService::new();

    rocket::build()
        .manage(platform_statistics_service)
        .mount("/", routes![index])
        .mount("/admin", admin_routes())
        .register("/", catchers![not_found])
}
