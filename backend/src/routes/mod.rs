use actix_web::web;

mod python_app;
mod ws;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/ws").route(web::get().to(ws::chat_route)));
}
