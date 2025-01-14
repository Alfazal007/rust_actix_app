use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web::Data,
    Error, HttpResponse,
};

use crate::{errors::GeneralError, AppState};

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<impl actix_web::body::MessageBody>, Error> {
    if req.cookie("accessToken").is_none() {
        let error_response = HttpResponse::Unauthorized().json(GeneralError {
            errors: "Unauthorized: Missing accessToken cookie".to_string(),
        });
        return Ok(req.into_response(error_response.map_into_boxed_body()));
    }

    let state = match req.app_data::<Data<AppState>>() {
        Some(data) => data,
        None => {
            let error_response = HttpResponse::InternalServerError().json(GeneralError {
                errors: "Failed to retrieve application state".to_string(),
            });
            return Ok(req.into_response(error_response.map_into_boxed_body()));
        }
    };

    let token = req.cookie("accessToken").unwrap().value().to_string();
    let token_eval_result =
        crate::tokens::validate_token::validate_token(&token, &state.access_secret);

    if token_eval_result.is_err() {
        let error_response = HttpResponse::Unauthorized().json(GeneralError {
            errors: token_eval_result.unwrap_err(),
        });
        return Ok(req.into_response(error_response.map_into_boxed_body()));
    }

    next.call(req).await
}
