use crate::rsp::Rsp;
use crate::service::HttpServiceState;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use vintage_msg::{BlockHeight, Proto};

pub(crate) fn build_router(state: HttpServiceState) -> Router {
    Router::new()
        .route("/sql_migrations", get(get_sql_migrations))
        .route("/id_hash_pairs", get(get_id_hash_pairs))
        .with_state(state)
}

#[derive(Deserialize)]
struct Param {
    proto: Proto,
    block_height_begin: BlockHeight,
    block_height_end: BlockHeight,
}

async fn get_sql_migrations(
    state: State<HttpServiceState>,
    Query(Param {
        proto,
        block_height_begin,
        block_height_end,
    }): Query<Param>,
) -> Json<serde_json::Value> {
    let rsp = match state
        .meta_data_db
        .query_sql_migration(proto, block_height_begin, block_height_end)
        .await
    {
        Ok(entities) => Rsp::success(entities),
        Err(err) => {
            log::error!("query_sql_migration error: {:?}", err);
            Rsp::failed()
        }
    };
    serde_json::to_value(rsp).unwrap().into()
}

async fn get_id_hash_pairs(
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
