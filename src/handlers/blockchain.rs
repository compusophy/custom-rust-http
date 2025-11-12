use axum::{extract::State, Json};

use crate::services::blockchain::get_current_block;
use crate::state::AppState;
use crate::types::BlockResponse;

pub async fn get_current_block_handler(State(state): State<AppState>) -> Json<BlockResponse> {
    if let Some(block_number) = *state.latest_block.read().await {
        Json(BlockResponse {
            block_number: format!("0x{:x}", block_number),
            block_number_decimal: block_number,
        })
    } else {
        match get_current_block().await {
            Ok(response) => response,
            Err(_error) => Json(BlockResponse {
                block_number: "error".to_string(),
                block_number_decimal: 0,
            }),
        }
    }
}


