use actix_web::{web, Error, HttpResponse};

use crate::services::ai_model::AIModel;
use log::error; // Add this import for the error! macro

pub async fn ingest_documents(ai_model: web::Data<AIModel>) -> Result<HttpResponse, Error> {
    ai_model.get_ref().ingest_documents().await.map_err(|e| {
        error!("Failed to ingest documents: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    Ok(HttpResponse::Ok().body("Documents ingested successfully"))
}

pub async fn query_documents(
    ai_model: web::Data<AIModel>,
    query: web::Json<String>,
) -> Result<HttpResponse, Error> {
    let response = ai_model
        .get_ref()
        .query_documents(query.into_inner())
        .await
        .map_err(|e| {
            error!("Failed to generate response: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    Ok(HttpResponse::Ok().body(response))
}
