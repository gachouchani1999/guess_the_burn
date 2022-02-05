use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Uint128, Storage, StdResult};
use cw_storage_plus::{Item, Map};
use cw_utils::{Expiration, Scheduled};


pub const BETS: Map<String, Vec<String>> = Map::new("bets");
pub const ADDRESS_BETS: Map<String, Vec<Uint128>> = Map::new("address_bets");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_bets: Uint128,
    pub total_jackpot: Uint128,
    pub cw20_token_address: String,
    pub community_pool_address: String,
    pub bet_start_block: Scheduled,
    pub bet_end_block: Expiration,
    pub burn_block: Expiration,
    pub bet_base_price: Uint128,
    pub bet_init_block: u64,
}

pub const STATE: Item<State> = Item::new("state");

pub fn save_bet(storage: &mut dyn Storage, supply: String, address: String) -> () {
    let vec_addresses = BETS.may_load(storage, supply).unwrap();
    let vec_addresses = match vec_addresses {
        None => {BETS.save(storage, supply, &vec![address])},
        Some(addresses) => {addresses.push(address);BETS.save(storage, supply, &addresses)}
    };

}


pub fn read_bet(storage: & dyn Storage, supply: String) -> StdResult<Vec<String>> {
    BETS.load(storage, supply) 
}

pub fn save_address_bet(storage: &mut dyn Storage, address: String, bet: Uint128) -> () {
    let vec_bets = ADDRESS_BETS.may_load(storage,address).unwrap();
    let vec_bets = match vec_bets {
        None => {ADDRESS_BETS.save(storage, address, &vec![bet])},
        Some(bets) => {bets.push(bet);ADDRESS_BETS.save(storage, address, &bets)}
    };

}


pub fn read_address_bet(storage: & dyn Storage, address: String) -> StdResult<Vec<Uint128>> {
    ADDRESS_BETS.load(storage, address) 
}

