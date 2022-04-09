#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, StdError};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, EntriesResponse};
use crate::state::{State, STATE, ENTRIES};
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // ExecuteMsg::StartGame { opponent } => try_start_game(deps, info, opponent.to_string()),
        ExecuteMsg::EnterRaffle { entering_address } => try_upsert_entry(deps, info, entering_address),
        // ExecuteMsg::UpdateOwner { owner } => try_update_owner(deps, info, owner),
    }
}

pub fn try_upsert_entry(
    deps: DepsMut,
    info: MessageInfo,
    entering_address: String,
) -> Result<Response, ContractError> {
    let entry_address = deps.api.addr_validate(&entering_address)?;
    let state = STATE.load(deps.storage)?;
    let increment_entry = | num_entries: Option< u8> | -> StdResult<u8> {
        match num_entries {
            Some(number) => Ok(number + 1),
            None => Ok(1),
        }
    };
    if entry_address != info.sender {
        if info.sender != state.owner {
            return Err(ContractError::WrongAddress {});
        } else {
            ENTRIES.update(deps.storage, entry_address, increment_entry)?;
        }
    } else {
        ENTRIES.update(deps.storage, entry_address, increment_entry)?;
    } 
    Ok(Response::new()
        .add_attribute("method", "enter_raffle")) 
}

// pub fn try_start_game(
//     deps: DepsMut,
//     info: MessageInfo,
//     opponent: String,
// ) -> Result<Response, ContractError> {
//     let checked: StdResult<Addr> = deps.api.addr_validate(&opponent);
//     match checked {
//         Ok(_) => {
//             Ok(Response::new().add_attribute("method", "start_game"))

//         }
//         Err(e) => Err(ContractError::InvalidLength{}),
//     }

// }

// pub fn try_update_owner(
//     deps: DepsMut,
//     info: MessageInfo,
//     owner: String,
// ) -> Result<Response, ContractError> {
//     STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
//         if info.sender != state.owner {
//             return Err(ContractError::Unauthorized {});
//         }
//         state.owner = Addr::unchecked(owner);
//         Ok(state)
//     })?;
//     Ok(Response::new().add_attribute("method", "update_owner"))
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetEntries { entry_address } => {
            let raw_entry = ENTRIES.load(deps.storage, entry_address)?;
            to_binary(&raw_entry)
        }
        // QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        // QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
    }
}

// fn query_count(deps: Deps) -> StdResult<CountResponse> {
//     let state = STATE.load(deps.storage)?;
//     Ok(CountResponse { count: state.count })
// }

// fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
//     let state = STATE.load(deps.storage)?;
//     Ok(OwnerResponse { owner: state.owner.to_string() })
// }

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization_and_enter_raffle() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { };
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // user can submit their address
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::EnterRaffle { entering_address: "creator".to_string() };
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetEntries { entry_address: Addr::unchecked("creator")}).unwrap();
        let value: u8 = from_binary(&res).unwrap();
        assert_eq!(1, value);

        //user cannot submit other peerson's addy
        let info = mock_info("not creator", &coins(2, "token"));
        let msg = ExecuteMsg::EnterRaffle { entering_address: "creator".to_string() };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            ContractError::WrongAddress {} => assert_eq!(true, true),
            e => panic!("Unexpected error: {}", e),
        }

        //admin can submit other person's addy
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::EnterRaffle { entering_address: "not creator".to_string() };
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetEntries { entry_address: Addr::unchecked("not creator")}).unwrap();
        let value: u8 = from_binary(&res).unwrap();
        assert_eq!(1, value);
    }


    // #[test]
    // fn proper_initialization_and_start_game() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // beneficiary can release it
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::StartGame { opponent: Addr::unchecked("a") };
    //     let res = execute(deps.as_mut(), mock_env(), info, msg);
    //     match res.unwrap_err() {
    //         ContractError::InvalidLength {} => assert_eq!(true, true),
    //         e => panic!("Unexpected error: {}", e),
    //     }

    //     // it worked, let's query the state
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::StartGame { opponent: Addr::unchecked("terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8")};
    //     let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(Response::new().add_attribute("method", "start_game"), res);
    // }

    // #[test]
    // fn update_owner() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should have owner of creator
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
    //     let value: OwnerResponse = from_binary(&res).unwrap();
    //     assert_eq!("creator", value.owner);

    //     // beneficiary can release it
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::UpdateOwner { owner: "anyone".to_string() };
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should have owner of anyone now
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
    //     let value: OwnerResponse = from_binary(&res).unwrap();
    //     assert_eq!("anyone", value.owner);
    // }
}
