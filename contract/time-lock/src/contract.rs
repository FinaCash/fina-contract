use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, Timestamp,
    Uint128
};

use secret_toolkit::snip20::{register_receive_msg, transfer_msg};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg};
use crate::state::{Config, Reward, BLOCK_SIZE, CONFIG_KEY, CLAIMED, REWARD_KEY};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api
        .debug(format!("Contract was initialized by {}", info.sender).as_str());

    let init_config = Config {
        asset_addr: deps.api.addr_validate(&msg.asset_addr).unwrap(),
        asset_hash: msg.asset_hash.clone(),
        admin: info.sender.clone(),
    };
    CONFIG_KEY.save(deps.storage, &init_config)?;

    let init_reward = Reward {
        recipient: deps.api.addr_validate(&msg.recipient).unwrap(),
        total_amount: msg.total_amount,
        start_ts: Timestamp::from_seconds(msg.start_ts),
        end_ts: Timestamp::from_seconds(msg.end_ts),
    };
    REWARD_KEY.save(deps.storage, &init_reward)?;

    CLAIMED.save(deps.storage, &Uint128::new(0))?;

    let response = Response::new()
        .add_message(register_receive_msg(
            env.contract.code_hash,
            None,
            BLOCK_SIZE,
            msg.asset_hash.clone(),
            msg.asset_addr,
        )?)
        .add_attribute("status", "success");
    Ok(response)
}

//-------------------------------------------- HANDLES ---------------------------------
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let res = match msg {
        ExecuteMsg::ClaimRewards { .. } => try_claim(deps, env, info),
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            ..
        } => receive(deps, env, info, sender, from, amount),
    };

    pad_handle_result(res, BLOCK_SIZE)
}

fn try_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    // ref: https://github.com/luminaryphi/fund-forwarding/blob/main/src/contract.rs
    let response = Response::new();

    let reward_config = REWARD_KEY.load(deps.storage)?;
    let config = CONFIG_KEY.load(deps.storage)?;
    let mut claimed = CLAIMED.load(deps.storage)?;

    // Check if it is the recipient that call the function
    if info.sender != reward_config.recipient {
        return Err(ContractError::Unauthorized {});
    }

    let now = env.block.time;

    // Check if recipient calls too early
    if now <= reward_config.start_ts {
        return Err(ContractError::TooEarlyPleaseChill {});
    }

    // For math, all numbers are strictly u64, so we don't need to
    // worry about rounding issue.
    let vested_amt = infer_vested_amt(
        now.seconds(),
        reward_config.start_ts.seconds(),
        reward_config.end_ts.seconds(),
        reward_config.total_amount,
    );

    let token_to_claim: Uint128;

    if vested_amt == Uint128::new(0) || vested_amt <= claimed {
        return Err(ContractError::TooQuickPleaseChill {})
    } else {
        token_to_claim = vested_amt - claimed;
    }
    let cosmos_msg = transfer_msg(
        reward_config.recipient.into_string(),
        token_to_claim,
        None,
        None,
        BLOCK_SIZE,
        config.asset_hash,
        config.asset_addr.into_string(),
    )?;
    

    claimed += token_to_claim;

    CLAIMED.save(deps.storage, &claimed)?;

    Ok(response.add_message(cosmos_msg))
}

fn receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {
    // goes with saying that this depend on the fn try_send{} from the snip20 contract
    // please don't use transfer 
    let reward_config = REWARD_KEY.load(deps.storage)?;
    let config = CONFIG_KEY.load(deps.storage)?;
    let claimed = CLAIMED.load(deps.storage)?;

    if info.sender != config.asset_addr {
        return Err(ContractError::WrongToken {});
    }

    if amount > (reward_config.total_amount - claimed) {
        return Err(ContractError::TooManyToken {});
    }

    Ok(Response::new()
        .add_attribute("num_token_receive", amount)
    )
}

// ---------------------------------------- QUERIES --------------------------------------
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let res = match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::UnlockStats {} => query_unlock_stats(deps),
    };

    pad_query_result(res, BLOCK_SIZE)
}

fn query_config(deps: Deps) -> Result<Binary, ContractError> {
    let config = CONFIG_KEY.load(deps.storage)?;
    Ok(to_binary(&QueryAnswer::ConfigResponse {config})?)
}

fn query_unlock_stats(deps: Deps) -> Result<Binary, ContractError> {
    let reward_config = REWARD_KEY.load(deps.storage)?;
    let claimed = CLAIMED.load(deps.storage)?;

    Ok(to_binary(&QueryAnswer::UnlockStatsResponse {
        start_ts: reward_config.start_ts.seconds(),
        end_ts: reward_config.end_ts.seconds(),
        total_amount: reward_config.total_amount,
        claimed_amount: claimed,
    })?)
}

// common function
pub fn infer_vested_amt(now: u64, start_s: u64, end_s: u64, amt: Uint128) -> Uint128 {
    let vested_amt: Uint128;

    if now >= end_s {
        vested_amt = amt;
    } else {
        let union_range = end_s - start_s;
        let vested_range = now - start_s;
        
        vested_amt = amt.checked_multiply_ratio(
            vested_range,
            union_range
        ).unwrap_or(Uint128::new(0));
    }

    vested_amt
}


#[cfg(test)]
mod tests {
    use crate::contract::{instantiate, query};
    use crate::msg::{InstantiateMsg, QueryMsg, QueryAnswer};
    use cosmwasm_std::testing::*;
    use cosmwasm_std::{from_binary, Addr, Uint128};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let info = mock_info(
            "creator",
            &[],
        );
        let init_msg = InstantiateMsg {
            asset_addr: Addr::unchecked("snip20_addr").to_string(),
            asset_hash: String::from("snip_hash"),
            recipient: Addr::unchecked("intend_recipient").to_string(),
            total_amount: Uint128::new(10000),
            start_ts: 1676041975,
            end_ts: 1676473975,
        };

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

        assert_eq!(1, res.messages.len());

        // // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::UnlockStats {}).unwrap();
        let values: QueryAnswer = from_binary(&res).unwrap();
        
        println!("{:?}", values);

        assert_eq!(QueryAnswer::UnlockStatsResponse {
            start_ts: 1676041975u64,
            end_ts: 1676473975u64,
            total_amount: Uint128::new(10000),
            claimed_amount: Uint128::new(0),
        }, values);
    }
}
