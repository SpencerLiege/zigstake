use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, Order, StdResult, Uint128};

use crate::state::{ Bet, Config, Round, CONFIG, LEADERBOARDENTRY, ROUNDS, USER_BETS};
use crate::msg::{LeaderboardEntry, QueryMsg};




pub fn query (
    deps: Deps,
    _env: Env,
    msg: QueryMsg
) -> StdResult<Binary> {
    match  msg {
        QueryMsg::GetRoundDetails { round_id } => {
            to_json_binary(&get_round_details(deps, round_id)?)
        },
        QueryMsg::GetAllRoundDetails {  } => {
            to_json_binary(&get_all_round_details(deps)?)
        },
        QueryMsg::GetUserPrediction { round_id, user } => {
            let addr = deps.api.addr_validate(&user)?;
            to_json_binary(&get_user_prediction(deps, round_id, addr)?)
        },
        QueryMsg::GetAllUserPredictions {  } => {
            to_json_binary(&get_all_user_prediction(deps)?)
        },
        QueryMsg::GetPool { round_id } => {
            to_json_binary(&get_pool(deps, round_id)?)
        },
        QueryMsg::IsPaused {  } => {
            let config: Config = CONFIG.load(deps.storage)?;
            to_json_binary(&config.paused)
        },
        QueryMsg::GetLeaderboard {  } => {
            to_json_binary(&get_leaderboard(deps)?)
        }
     } 
}


fn get_round_details(deps: Deps, round_id: u64) -> StdResult<Round> {
    let round: Round = ROUNDS.load(deps.storage, round_id)?;

    Ok(round)
}

fn get_all_round_details(deps: Deps) -> StdResult<Vec<Round>> {
    let rounds: StdResult<Vec<_>> = ROUNDS
    .range(deps.storage, None, None, Order::Ascending)
    .map(|item| {
        let (_, round) = item?;
        Ok(round)
    })
    .collect();

    rounds
}

fn get_user_prediction(deps: Deps, round_id: u64, user: Addr) -> StdResult<Bet> {
    let bet: Bet = USER_BETS.load(deps.storage, (round_id, &user))?;

    Ok(bet)
}

fn get_all_user_prediction(deps: Deps) -> StdResult<Vec<Bet>> {
    let bets: StdResult<Vec<_>> = USER_BETS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, bet) = item?;
            Ok(bet)
        })
        .collect();
    bets
}

fn get_pool(deps:Deps, round_id: u64) -> StdResult<Uint128> {
    let round: Round = ROUNDS.load(deps.storage, round_id)?;

    Ok(round.total_pool)
}

fn get_leaderboard(deps: Deps) -> StdResult<Vec<LeaderboardEntry>> {
    let boards: StdResult<Vec<_>> = LEADERBOARDENTRY
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (_, lb) = item?;
            Ok(lb)
        })
        .collect();
    boards
}