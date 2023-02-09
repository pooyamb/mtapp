use seaqs::filters::UuidFilterSet;
use utoipa::{
    OpenApi, 
};

use crate::{admin, errors::SessionErrorOai, handlers, models::Session, schemas::SessionList};


#[derive(OpenApi)]
#[openapi(
    info(description = "Session management endpoints"),
    paths(handlers::list, handlers::get, handlers::delete),
    components(schemas(
        // Response
        Session, SessionList,
        
        // Errors
        SessionErrorOai::NotFound
    ))
)]
pub(crate) struct PublicSessionOpenApi;

#[derive(OpenApi)]
#[openapi(
    info(description = "Session management endpoints"),
    paths(admin::list, admin::batch_delete, admin::get, admin::delete),
    components(schemas(
        // Response
        Session, SessionList,
        
        // Params
        UuidFilterSet,
        
        // Errors
        SessionErrorOai::NotFound
    ))
)]
pub(crate) struct InternalSessionOpenApi;
