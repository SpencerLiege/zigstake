use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, StdError};
use crate::msg::InstantiateMsg;
use crate::state::{ CONFIG, Config};

pub fn instantiate( 
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> StdResult<Response> {
    // validate treasury fees: must be <= 10000 (100%) ..... we use 5%
    if msg.treasury_fee > 10_000 {
        return  Err(StdError::generic_err("Treasury fee must be <= 10000 basis points"));
    }

    let config = Config {
        admin: info.sender.clone(),
        treasury_fee: msg.treasury_fee,
        paused: false,
        current_round_id: 1
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("treasury-fee", msg.treasury_fee.to_string()))

}