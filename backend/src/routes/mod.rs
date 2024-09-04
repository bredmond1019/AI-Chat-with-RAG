use actix_web::web;

mod python_app;
mod ws;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/ws").route(web::get().to(ws::chat_route)));
    cfg.service(web::resource("/ingest").route(web::post().to(python_app::ingest_documents)));
    cfg.service(web::resource("/query").route(web::post().to(python_app::query_documents)));
}
