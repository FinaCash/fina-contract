use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use secret_toolkit::storage::Item;

pub static CONFIG_KEY: Item<Config> = Item::new(b"config");
pub static REWARD_KEY: Item<Reward> = Item::new(b"reward");
pub static CLAIMED: Item<Uint128> = Item::new(b"claimed");
/// pad handle responses and log attributes to blocks of 256 bytes to prevent leaking info based on
/// response size
pub const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub asset_addr: Addr,
    pub asset_hash: String,
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Reward {
    pub recipient: Addr,
    pub total_amount: Uint128,
    pub start_ts: Timestamp,
    pub end_ts: Timestamp,
}
