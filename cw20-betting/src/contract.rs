#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg, BankMsg, Coin, QueryRequest, WasmQuery, Order};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfoResponse, TotalBetsResponse, JackpotResponse, BetPriceResponse, AddressBetResponse};
use crate::state::{State, STATE, BETS, read_bet, save_bet, ADDRESS_BETS, read_address_bet, save_address_bet};
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
            msg: to_binary(&QueryMsg::TokenInfo {})?,
        }))?;
    
    let total_supply = token_info.total_supply;
    let winners = BETS.may_load(deps.storage, total_supply.to_string())?;
    let winners = match winners {
        None => vec![],
        Some(vec_winners) => vec_winners
    };
    // If no winners send back to community pool
    let resp = Response::new().add_attribute("method", "announce");
    if winners.is_empty() {
        resp.add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: state.community_pool_address,
            amount: vec![Coin { denom: "ujunox".to_string() , amount: state.total_jackpot }],
        }));
    }
    let winners_length : u64 = winners.len() as u64;
    let ratio_per_winner = 100/winners_length;
    for winner in winners {
        resp.add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: winner,
            amount: vec![Coin { denom: "ujunox".to_string() , amount: state.total_jackpot.multiply_ratio(ratio_per_winner, 100u128 ) }],
        }));
    }
    Ok(resp)
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

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}