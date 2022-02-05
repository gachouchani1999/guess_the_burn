#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg, BankMsg, Coin, QueryRequest, WasmQuery, Order};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfoResponse, TotalBetsResponse, JackpotResponse, BetPriceResponse, AddressBetResponse, CW20Query};
use crate::state::{State, STATE, BETS, read_bet, save_bet, ADDRESS_BETS, read_address_bet, save_address_bet};
use cw_utils::{Expiration, Scheduled};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-betting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        total_bets: Uint128::from(0u64),
        total_jackpot: Uint128::from(0u64),
        cw20_token_address: msg.cw20_token_address,
        community_pool_address: msg.community_pool_address,
        bet_start_block: msg.bet_start_block,
        bet_end_block: msg.bet_end_block,
        burn_block: msg.burn_block,
        bet_base_price: msg.bet_base_price,
        bet_init_block: env.block.height,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("total_bets", state.total_bets)
        .add_attribute("total_jackpot", state.total_jackpot)
        .add_attribute("cw20_token_address", state.cw20_token_address)
        .add_attribute("community_pool", state.community_pool_address)  
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bet {choosed_number} => execute_bet(deps, env, info, choosed_number),
        ExecuteMsg::AnnounceWinner {  } => execute_announce(deps, env, info),
    }
}

pub fn execute_bet(deps: DepsMut, env:Env, info: MessageInfo, choosed_number:Uint128) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    if !state.bet_start_block.is_triggered(&env.block) || state.bet_end_block.is_expired(&env.block) {
        return Err(ContractError::UnallowedTime{})
    }
    let elapsed_blocks = env.block.height - state.bet_init_block;
    let price = (Uint128::from(elapsed_blocks) + (state.total_bets.checked_div(Uint128::from(40u64)).unwrap())) + state.bet_base_price;

    if let Some(coins) = info.funds.first() {
        if coins.denom != "ujunox" || coins.amount < price {
            return Err(ContractError::NoFunds{})
        }
    } else {
        return Err(ContractError::NoFunds {})
    }

    
    save_bet(deps.storage, choosed_number.to_string(), info.sender.to_string());
    save_address_bet(deps.storage, info.sender.to_string(), choosed_number);
    state.total_bets += Uint128::from(1u64);
    let mut jackpot_amount = info.funds.first().unwrap().amount.checked_mul(Uint128::from(80u64)).unwrap();
    jackpot_amount = jackpot_amount.checked_div(Uint128::from(100u64)).unwrap();

    let community_pool_amount = info.funds.first().unwrap().amount.checked_sub(jackpot_amount).unwrap();

    state.total_jackpot += jackpot_amount;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "execute_bet")
    .add_message(CosmosMsg::Bank(BankMsg::Send {
        to_address: state.community_pool_address,
        amount: vec![Coin { denom: "ujunox".to_string() , amount: community_pool_amount }],
    }))
    

)
}


pub fn execute_announce(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
     let state = STATE.load(deps.storage)?;
     if !state.burn_block.is_expired(&env.block) {
         return Err(ContractError::BurnBlock {})
     }
     // query total of cw20 smart contract
     let token_info = deps.querier
        .query::<TokenInfoResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: state.cw20_token_address,
            msg: to_binary(&CW20Query::TokenInfo {})?,
        }))?;
    
    let total_supply = token_info.total_supply;
    let winners = BETS.may_load(deps.storage, total_supply.to_string())?;
    let winners = match winners {
        None => vec![],
        Some(vec_winners) => vec_winners
    };
    // If no winners send back to community pool
    let mut resp: Vec<CosmosMsg> = vec![];

    let winners_length : u64 = winners.len() as u64;
    let ratio_per_winner = 100/winners_length;
    for winner in winners {
        resp.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: winner,
            amount: vec![Coin { denom: "ujunox".to_string() , amount: state.total_jackpot.multiply_ratio(ratio_per_winner, 100u128 ) }],
        }));
    }
    Ok(Response::new()
        .add_messages(resp)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TotalBets {} => to_binary(&query_total_bets(deps)?),
        QueryMsg::Jackpot {} => to_binary(&query_jackpot(deps)?),
        QueryMsg::BetPrice {} => to_binary(&query_bet_price(deps, env)?),
        QueryMsg::AddressBet {address} => to_binary(&query_address_bet(deps, address)?),
        

    }
}

fn query_total_bets(deps: Deps) -> StdResult<TotalBetsResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(TotalBetsResponse { total_bets: state.total_bets })
}

fn query_jackpot(deps: Deps) -> StdResult<JackpotResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(JackpotResponse { jackpot: state.total_jackpot })
}

fn query_bet_price(deps: Deps, env: Env) -> StdResult<BetPriceResponse> {
    let state = STATE.load(deps.storage)?;
    let elapsed_blocks = env.block.height - state.bet_init_block;
    let price = (Uint128::from(elapsed_blocks) + (state.total_bets.checked_div(Uint128::from(40u64)).unwrap())) + state.bet_base_price;
    Ok(BetPriceResponse { bet_price: price })
}

fn query_address_bet(deps: Deps, address: String) -> StdResult<AddressBetResponse> {
    let bets = read_address_bet(deps.storage, address)?;
    Ok(AddressBetResponse { bet: bets })
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { 
            cw20_token_address: "address_token".to_string(),
            bet_start_block: Scheduled::AtHeight(10000),
            bet_end_block: Expiration::AtHeight(20000),
            burn_block: Expiration::AtHeight(30000),
            community_pool_address: "community_address".to_string(),
            bet_base_price: Uint128::from(100000u128)
         };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }
    #[test]
    fn query_bet_price_and_bet() {
        let mut deps = mock_dependencies_with_balance(&[]);

        let msg = InstantiateMsg { 
            cw20_token_address: "address_token".to_string(),
            bet_start_block: Scheduled::AtHeight(10000),
            bet_end_block: Expiration::AtHeight(20000),
            burn_block: Expiration::AtHeight(30000),
            community_pool_address: "community_address".to_string(),
            bet_base_price: Uint128::from(100000u128)
         };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        // query price
        let query_msg = QueryMsg::BetPrice{};
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let price: BetPriceResponse = from_binary(&query_res).unwrap();
        let msg = ExecuteMsg::Bet {choosed_number: Uint128::from(120000000000u128)};
        let info = mock_info("bettor", &coins(u128::from(price.bet_price), "ujunox"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        // query user bets, total jackpot, total bets
        let query_msg = QueryMsg::AddressBet{address: "bettor".to_string()};
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let address_bet: AddressBetResponse = from_binary(&query_res).unwrap();
        assert_eq!(address_bet.bet, vec![Uint128::from(120000000000u128)]);

        let query_msg = QueryMsg::Jackpot {};
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let jackpot: JackpotResponse = from_binary(&query_res).unwrap();
        assert_ne!(jackpot.jackpot, Uint128::from(0u64));

        let query_msg = QueryMsg::TotalBets{};
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let jackpot: TotalBetsResponse = from_binary(&query_res).unwrap();
        assert_eq!(jackpot.total_bets, Uint128::from(1u64));

        // another bettor 
        let msg = ExecuteMsg::Bet {choosed_number: Uint128::from(120000000000u128)};
        let info = mock_info("another_bettor", &coins(u128::from(price.bet_price), "ujunox"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        let query_msg = QueryMsg::TotalBets{};
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let jackpot: TotalBetsResponse = from_binary(&query_res).unwrap();
        assert_eq!(jackpot.total_bets, Uint128::from(2u64));

        let query_msg = QueryMsg::Jackpot {};
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let jackpot: JackpotResponse = from_binary(&query_res).unwrap();
        assert_ne!(jackpot.jackpot, Uint128::from(0u64));

        // same bettor
        let msg = ExecuteMsg::Bet {choosed_number: Uint128::from(121000000000u128)};
        let info = mock_info("another_bettor", &coins(u128::from(price.bet_price), "ujunox"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        // query their bets
        let query_msg = QueryMsg::AddressBet{address: "another_bettor".to_string()};
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let address_bet: AddressBetResponse = from_binary(&query_res).unwrap();
        assert_eq!(address_bet.bet, vec![Uint128::from(120000000000u128),Uint128::from(121000000000u128)]);
    }
}
