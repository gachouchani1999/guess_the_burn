use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use cw20_betting::msg::{CW20Query, ExecuteMsg, InstantiateMsg, QueryMsg, TotalBetsResponse, JackpotResponse, AddressBetResponse, BetPriceResponse};
use cw20_betting::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(CW20Query), &out_dir);
    export_schema(&schema_for!(TotalBetsResponse), &out_dir);
    export_schema(&schema_for!(JackpotResponse), &out_dir);
    export_schema(&schema_for!(AddressBetResponse), &out_dir);
    export_schema(&schema_for!(BetPriceResponse), &out_dir);

}
