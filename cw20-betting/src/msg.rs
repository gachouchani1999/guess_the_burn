use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128};
use cw_utils::{Expiration, Scheduled};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub cw20_token_address: String,
    pub bet_start_block: Scheduled,
    pub bet_end_block: Expiration,
    pub burn_block: Expiration,
    pub community_pool_address: String,
    pub bet_base_price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Bet {choosed_number: Uint128},
    AnnounceWinner {},

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    TotalBets {},
    Jackpot {},
    AddressBet {address: String}, 
    BetPrice {},

    // For CW20 contract
    TokenInfo {}
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalBetsResponse {
    pub total_bets: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct JackpotResponse {
    pub jackpot: Uint128,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddressBetResponse {
    pub bet: Vec<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BetPriceResponse {
    pub bet_price: Uint128,
}

// For CW20 Query
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenInfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}