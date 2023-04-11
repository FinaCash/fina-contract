use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Binary};
use crate::state::Config;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstantiateMsg {
    pub asset_addr: String,
    pub asset_hash: String,
    pub recipient: String,
    pub total_amount: Uint128,
    pub start_ts: u64,
    pub end_ts: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ClaimRewards {},
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    UnlockStats {},
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    ConfigResponse {
        config: Config
    },
    UnlockStatsResponse {
        start_ts: u64,
        end_ts: u64,
        total_amount: Uint128,
        claimed_amount: Uint128,
    }
}