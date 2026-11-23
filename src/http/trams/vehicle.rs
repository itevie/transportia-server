use axum::Json;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::auth::quick::{is_admin, is_trusted_user};
use crate::db::resource::tram::tram::{TramQuery, Trams};
use crate::db::resource::tram::tram_def::{Tram, TramFilledOut};
use crate::extractors::authenticator::AuthUser;
use crate::extractors::optional_authenticator::OptionalAuthUser;
use crate::extractors::ratelimit::{CreateRatelimit, DeleteRatelimit, QueryRatelimit, RateLimit, SearchRatelimit, UpdateRatelimit};
use crate::http::HttpState;
use crate::http::errors::ApiError;

#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TramSearchQuery {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramCreateRequest {
    pub operator_id: Option<i64>,
    pub fleet: String,
    pub livery_id: Option<i64>,
    pub tram_type_id: Option<i64>,
    pub name: Option<String>,
    pub branding: Option<String>,
    pub garage: Option<String>,
    pub withdrawn: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramUpdateRequest {
    pub operator: Option<i64>,
    pub fleet: Option<String>,
    pub livery: Option<i64>,
    pub tram_type: Option<i64>,
    pub name: Option<String>,
    pub branding: Option<String>,
    pub garage: Option<String>,
    pub withdrawn: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramCreateResponse {
    pub id: i64,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TramBulkUpload {
    pub trams: Vec<TramCreateRequest>,
}

#[derive(Debug, Deserialize)]
struct TramCsvRow {
    fleet: String,
    operator_id: Option<i64>,
    tram_type_id: Option<i64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    withdrawn: String,
    #[serde(default)]
    garage: Option<String>,
    #[serde(default)]
    branding: Option<String>,
}

impl From<TramCsvRow> for Tram {
    fn from(row: TramCsvRow) -> Self {
        Self {
            id: 0,
            fleet: row.fleet,
            operator_id: row.operator_id,
            livery_id: None,
            tram_type_id: row.tram_type_id,
            name: row.name.filter(|x| !x.is_empty()),
            withdrawn: matches!(row.withdrawn.to_lowercase().as_str(), "true" | "1"),
            garage: row.garage.filter(|x| !x.is_empty()),
            branding: row.branding.filter(|x| !x.is_empty()),
        }
    }
}

#[utoipa::path(
    get,
    path = "/trams",
    tag = "Trams",
    summary = "Search trams",
    params(TramQuery),
    responses(
        (
            status = 200,
            description = "Trams matching the search query",
            body = [TramFilledOut]
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn search_trams(
    State(state): State<HttpState>,
    Query(query): Query<TramQuery>,
    _ratelimit: RateLimit<SearchRatelimit>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Result<Json<Vec<TramFilledOut>>, ApiError> {
    let trams = state
        .db
        .trams
        .search(&query)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    let filled_out = Trams::fill_out(state.db.clone(), trams, user.map(|x| x.user_id))
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out))
}

#[utoipa::path(
    get,
    path = "/trams/{id}",
    tag = "Trams",
    summary = "Get tram",
    params(
        ("id" = i64, Path, description = "Tram ID")
    ),
    responses(
        (
            status = 200,
            description = "Tram",
            body = TramFilledOut
        ),
        (
            status = 404,
            description = "Tram not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    )
)]
pub async fn get_tram(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<QueryRatelimit>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Result<Json<TramFilledOut>, ApiError> {
    let tram = state
        .db
        .trams
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram not found".to_string()))?;

    let filled_out = Trams::fill_out(state.db.clone(), vec![tram], user.map(|x| x.user_id))
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out.get(0).unwrap().clone()))
}

#[utoipa::path(
    post,
    path = "/trams",
    tag = "Trams",
    summary = "Create a tram",
    request_body = TramCreateRequest,
    responses(
        (
            status = 201,
            description = "Tram created successfully",
            body = TramCreateResponse
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_tram(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    _ratelimit: RateLimit<CreateRatelimit>,
    Json(body): Json<TramCreateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let tram = Tram {
        id: 0,
        operator_id: body.operator_id,
        fleet: body.fleet,
        livery_id: body.livery_id,
        tram_type_id: body.tram_type_id,
        name: body.name,
        garage: body.garage,
        withdrawn: body.withdrawn,
        branding: body.branding,
    };

    let id = state
        .db
        .trams
        .create(&tram)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok((StatusCode::CREATED, Json(TramCreateResponse { id })).into_response())
}

#[utoipa::path(
    patch,
    path = "/trams/{id}",
    tag = "Trams",
    summary = "Update a tram",
    params(
        ("id" = i64, Path, description = "Tram ID")
    ),
    request_body = TramUpdateRequest,
    responses(
        (
            status = 204,
            description = "Tram updated successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 404,
            description = "Tram not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_tram(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<UpdateRatelimit>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<TramUpdateRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut tram = state
        .db
        .trams
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram not found".to_string()))?;

    if let Some(operator) = body.operator {
        tram.operator_id = Some(operator);
    }

    if let Some(fleet) = body.fleet {
        tram.fleet = fleet;
    }

    if let Some(livery) = body.livery {
        tram.livery_id = Some(livery);
    }

    if let Some(tram_type) = body.tram_type {
        tram.tram_type_id = Some(tram_type);
    }

    if let Some(name) = body.name {
        tram.name = Some(name);
    }

    if let Some(branding) = body.branding {
        tram.branding = Some(branding);
    }

    if let Some(withdrawn) = body.withdrawn {
        tram.withdrawn = withdrawn;
    }

    if let Some(garage) = body.garage {
        tram.garage = Some(garage);
    }

    state
        .db
        .trams
        .update(&tram)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[utoipa::path(
    delete,
    path = "/trams/{id}",
    tag = "Trams",
    summary = "Delete a tram",
    params(
        ("id" = i64, Path, description = "Tram ID")
    ),
    responses(
        (
            status = 204,
            description = "Tram deleted successfully"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 404,
            description = "Tram not found"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_tram(
    State(state): State<HttpState>,
    Path(id): Path<i64>,
    _ratelimit: RateLimit<DeleteRatelimit>,
    AuthUser { user_id }: AuthUser,
) -> Result<Response, ApiError> {
    is_admin(state.db.clone(), user_id).await?;

    state
        .db
        .trams
        .get(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Tram not found".to_string()))?;

    state
        .db
        .trams
        .delete(id)
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(StatusCode::NO_CONTENT.into_response())
}


#[derive(Deserialize, utoipa::ToSchema)]
pub struct TramCsvUploadRequest {
    pub csv: String,
}

#[utoipa::path(
    post,
    path = "/trams/csv",
    tag = "Trams",
    summary = "Preview a tram CSV import",
    request_body = TramCsvUploadRequest,
    responses(
        (
            status = 200,
            description = "Parsed tram objects",
            body = [TramFilledOut]
        ),
        (
            status = 400,
            description = "Invalid CSV"
        ),
        (
            status = 401,
            description = "Authentication required"
        ),
        (
            status = 403,
            description = "User is not trusted"
        ),
        (
            status = 500,
            description = "Internal server error"
        )
    ),
    security(("bearer_auth" = []))
)]
pub async fn preview_tram_csv(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    _ratelimit: RateLimit<CreateRatelimit>,
    Json(request): Json<TramCsvUploadRequest>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(request.csv.as_bytes());

    let mut trams = Vec::new();

    for (index, result) in reader.deserialize::<TramCsvRow>().enumerate() {
        let row = match result {
            Ok(row) => row,
            Err(err) => {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    format!("Invalid CSV row {}: {}", index + 2, err),
                )
                    .into_response());
            }
        };

        if row.fleet.is_empty() {
            return Ok((
                StatusCode::BAD_REQUEST,
                format!("Fleet number is missing on CSV row {}", index + 2),
            )
                .into_response());
        }

        trams.push(Tram::from(row));
    }

    if trams.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "The CSV contains no trams",
        )
            .into_response());
    }

    let filled_out = Trams::fill_out(
        state.db.clone(),
        trams,
        Some(user_id),
    )
        .await
        .map_err(|err| ApiError::Database(err.to_string()))?;

    Ok(Json(filled_out).into_response())
}
#[utoipa::path(
    post,
    path = "/trams/bulk",
    tag = "Trams",
    summary = "Upload Trams in bulk",
    request_body = TramBulkUpload,
    responses(
        (status = 201, description = "Trams imported successfully", body = usize),
        (status = 400, description = "Invalid import"),
        (status = 401, description = "Authentication required"),
        (status = 403, description = "User is not trusted"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn tram_bulk_upload(
    State(state): State<HttpState>,
    AuthUser { user_id }: AuthUser,
    _ratelimit: RateLimit<CreateRatelimit>,
    Json(body): Json<TramBulkUpload>,
) -> Result<Response, ApiError> {
    is_trusted_user(state.db.clone(), user_id).await?;

    if body.trams.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            "No trams provided",
        ).into_response());
    }

    let mut created_count = 0;

    for body in body.trams {
        let tram = Tram {
            id: 0,
            operator_id: body.operator_id,
            fleet: body.fleet,
            livery_id: body.livery_id,
            tram_type_id: body.tram_type_id,
            name: body.name,
            garage: body.garage,
            withdrawn: body.withdrawn,
            branding: body.branding,
        };

        let id = state
            .db
            .trams
            .upsert(&tram)
            .await
            .map_err(|err| ApiError::Database(err.to_string()))?;

        if id != 0 {
            created_count += 1;
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(created_count),
    ).into_response())
}