use crate::rsp::Rsp;
use crate::service::HttpServiceState;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use vintage_meta_data::{read_entities_csv, read_sql_migrations_csv};
use vintage_msg::{BlockHeight, Proto};

pub(crate) fn build_router(state: HttpServiceState) -> Router {
    Router::new()
        .route("/sql_migrations", get(sql_migrations_json))
        .route("/sql_migrations_csv", get(sql_migrations_csv))
        .route("/id_hash_pairs", get(id_hash_pairs_json))
        .route("/id_hash_pairs_csv", get(id_hash_pairs_csv))
        .with_state(state)
}

#[derive(Deserialize)]
struct Param {
    proto: Proto,
    block_height_begin: BlockHeight,
    block_height_end: BlockHeight,
}

async fn sql_migrations_json(
    state: State<HttpServiceState>,
    Query(Param {
        proto,
        block_height_begin,
        block_height_end,
    }): Query<Param>,
) -> Json<serde_json::Value> {
    let rsp = match state
        .meta_data_db
        .query_sql_migrations(proto, block_height_begin, block_height_end)
        .await
    {
        Ok(sql_migrations) => Rsp::success(sql_migrations),
        Err(err) => {
            log::error!("query_sql_migration error: {:?}", err);
            Rsp::failed()
        }
    };
    serde_json::to_value(rsp).unwrap().into()
}

async fn sql_migrations_csv(
    state: State<HttpServiceState>,
    Query(Param {
        proto,
        block_height_begin,
        block_height_end,
    }): Query<Param>,
) -> axum::response::Response {
    match read_sql_migrations_csv(
        &state.meta_data_db,
        proto,
        block_height_begin,
        block_height_end,
    )
    .await
    {
        Ok(csv) => csv_response("sql_migrations.csv", csv).into(),
        Err(err) => {
            log::error!("query_sql_migration error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn id_hash_pairs_json(
    state: State<HttpServiceState>,
    Query(Param {
        proto,
        block_height_begin,
        block_height_end,
    }): Query<Param>,
) -> Json<serde_json::Value> {
    let rsp = match state
        .meta_data_db
        .query_entities(proto, block_height_begin, block_height_end)
        .await
    {
        Ok(entities) => Rsp::success(entities),
        Err(err) => {
            log::error!("query_entities error: {:?}", err);
            Rsp::failed()
        }
    };
    serde_json::to_value(rsp).unwrap().into()
}

async fn id_hash_pairs_csv(
    state: State<HttpServiceState>,
    Query(Param {
        proto,
        block_height_begin,
        block_height_end,
    }): Query<Param>,
) -> axum::response::Response {
    match read_entities_csv(
        &state.meta_data_db,
        proto,
        block_height_begin,
        block_height_end,
    )
    .await
    {
        Ok(csv) => csv_response("id_hash_pairs.csv", csv).into(),
        Err(err) => {
            log::error!("query_sql_migration error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn csv_response(filename: &str, csv_content: String) -> axum::response::Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("text/csv").unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
    );
    (headers, csv_content).into_response()
}
