#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::instantiate::instantiate as contract_instance;
use crate::execute::execute as contract_executions;
use crate::query::query as contract_queries;


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:zigstake";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {

    // Set contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Initialize the contract
    contract_instance(deps, env, info, msg).map_err(ContractError::Std)

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    contract_executions(deps, info, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps, 
    env: Env, 
    msg: QueryMsg
) -> StdResult<Binary> {
    contract_queries(deps, env, msg)
}

// Contract test will go here and will be update
#[cfg(test)]
mod tests {

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info};
    use cosmwasm_std::{attr, coins, from_json, Addr, Uint128};

    use crate::msg::{self, ExecuteMsg, InstantiateMsg};
    use crate::state::CONFIG;
    
    fn inst(deps: DepsMut, addr: &Addr ) {
        let msg = InstantiateMsg {
            treasury_fee: 200
        };
        let env = mock_env();
        let info = message_info(&addr, &[]);

        instantiate(deps, env, info, msg).unwrap();
    }

    #[test]
    fn test_instantiate_contract () {
        let mut deps = mock_dependencies();
        let addr = deps.api.addr_make("creator");

        let instantiate_msg: InstantiateMsg = InstantiateMsg {
            treasury_fee: 500
        };
        
        let info = message_info(&addr, &[]);
        let env = mock_env();

        let response = instantiate(deps.as_mut(), env, info.clone(), instantiate_msg).unwrap();

        assert_eq!(
            response.attributes,
            vec![
                attr("action", "instantiate"),
                attr("admin", info.sender.to_string()),
                attr("treasury-fee", "500")
            ]
        );

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.admin, info.sender);
        assert_eq!(config.treasury_fee, 500);
        assert_eq!(config.paused, false);
        assert_eq!(config.current_round_id, 1);
    }

    #[test]
    fn test_place_bet() {
        // Instantiate the contract
        let mut deps = mock_dependencies();
        let addr= deps.api.addr_make("creator");
        inst(deps.as_mut(), &addr);

        // create user to place bet
        let bettor = deps.api.addr_make("bettor");
        let env = mock_env();
        let info = message_info(&bettor, &coins(20, "uzig"));
        let into = message_info(&addr, &[]);
        // let config = CONFIG.load(deps.storage).unwrap();

        let msg = ExecuteMsg::PlaceBet { round_id: 1, direction: msg::Direction::Down };
        let start_msg = ExecuteMsg::StartRound { price: Uint128::from(123456u64) };

        // call the contract
        execute(deps.as_mut(), env.clone(), into.clone(), start_msg).unwrap();
        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Assert values
        assert_eq!(
            response.attributes,
            vec![
                attr("action", "place_bet"),
                attr("round_id", "1"),
                attr("user", info.sender.to_string()),
                attr("amount", "20"),
                attr("direction", "Down")
            ]
        )
    }

}
