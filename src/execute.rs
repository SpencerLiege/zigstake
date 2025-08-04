use crate::error::ContractError;
use crate::msg::{Direction, ExecuteMsg, LeaderboardEntry};
use crate::state::{Bet, Config, Round, CONFIG, LEADERBOARDENTRY, ROUNDS, USER_BETS};
use cosmwasm_std::{
    BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, Uint128,
};

pub fn execute(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // <match the input message to the execute function
    match msg {
        ExecuteMsg::StartRound { price } => execute_start_round(deps, env, info, price),
        ExecuteMsg::LockRound { price } => execute_lock_round(deps, info, env, price),
        ExecuteMsg::EndRound { price } => execute_end_round(deps, env, info, price),
        ExecuteMsg::Pause {} => execute_pause(deps, env, info),
        ExecuteMsg::Resume {} => execute_resume(deps, env, info),
        ExecuteMsg::Withdraw { amount, recipient } => {
            execute_withdraw(deps, env, info, amount, recipient)
        }
        ExecuteMsg::PlaceBet {
            round_id,
            direction,
        } => execute_place_bet(deps, env, info, round_id, direction),
        ExecuteMsg::ClaimReward { round_id } => execute_claim_reward(deps, env, info, round_id),
    }
    // Ok(Response)
}

// ADMINN EXECUTE FUNCTIONS
fn execute_start_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    price: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Timestaps
    let beginning: Timestamp = env.block.time;
    let lock: Timestamp = Timestamp::from_seconds(300);
    let end: Timestamp = Timestamp::from_seconds(300);

    let round: Round = Round {
        id: config.current_round_id,
        bull_pool: Uint128::zero(),
        bear_pool: Uint128::zero(),
        total_pool: Uint128::zero(),
        start_time: beginning,
        lock_time: lock,
        end_time: end,
        // replace start price with actual value
        start_price: price,
        lock_price: Uint128::zero(),
        end_price: Uint128::zero(),
        result: None,
        executed: false,
        participants: vec![],
    };

    // Validate rounds
    if config.current_round_id == 1 {
        ROUNDS.save(deps.storage, config.current_round_id, &round)?;
    } else {
        let previous_round = config.current_round_id - 1;
        let prev_round: Round = ROUNDS.load(deps.storage, previous_round)?;

        if config.current_round_id > 1 && prev_round.end_price != Uint128::zero() {
            ROUNDS.save(deps.storage, config.current_round_id, &round)?;
        } else {
            return Err(ContractError::CannotStartNewRound {});
        }
    }

    Ok(Response::new()
        .add_attribute("action", "start_round")
        .add_attribute("round_id", config.current_round_id.to_string()))
}

fn execute_lock_round(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    price: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let round: Round = ROUNDS.load(deps.storage, config.current_round_id)?;
    if round.start_time == Timestamp::from_seconds(0) {
        return Err(ContractError::RoundNotStarted {});
    }

    if env.block.time < round.lock_time {
        return Err(ContractError::CannotLockBeforeTime {});
    }

    ROUNDS.update(
        deps.storage,
        config.current_round_id,
        |round| -> StdResult<_> {
            let mut r: Round = round.ok_or(StdError::generic_err("Round not found"))?;
            r.lock_price = price;
            Ok(r)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "lock_round")
        .add_attribute("round_id", config.current_round_id.to_string()))
}

fn execute_end_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    price: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let round: Round = ROUNDS.load(deps.storage, config.current_round_id)?;
    if round.lock_price == Uint128::zero() {
        return Err(ContractError::CannnotEndWithoutLockPrice {});
    }

    if env.block.time < round.end_time {
        return Err(ContractError::CannotEndBeforeTime {});
    }

    ROUNDS.update(
        deps.storage,
        config.current_round_id,
        |round| -> StdResult<_> {
            let mut r: Round = round.ok_or(StdError::generic_err("Round not found"))?;
            r.end_price = price;
            if r.lock_price > r.end_price {
                r.result = Some(Direction::Down);
            } else if r.lock_price > r.end_price {
                r.result = Some(Direction::Up);
            } else {
                r.result = None
            }
            r.executed = true;
            Ok(r)
        },
    )?;

    let round: Round = ROUNDS.load(deps.storage, config.current_round_id)?;

    for user in round.participants.iter() {
        let user_bet: Bet = USER_BETS.load(deps.storage, (config.current_round_id, user))?;
        let round: Round = ROUNDS.load(deps.storage, config.current_round_id)?;

        // Update the user leaderboard details
        LEADERBOARDENTRY.update(deps.storage, user, |board| -> StdResult<_> {
            let mut b: LeaderboardEntry =
                board.ok_or(StdError::generic_err("User data not found"))?;

            // seed reward
            let winning_pool: Uint128 = match round.result {
                Some(Direction::Up) => round.bull_pool,
                Some(Direction::Down) => round.bear_pool,
                None => return Err(StdError::generic_err("Round not ended")),
            };
            let losing_pool: Uint128 = round.total_pool - winning_pool;
            let reward: Uint128 = (user_bet.amount * losing_pool) / winning_pool + user_bet.amount;

            if round.result == Some(Direction::Up) && user_bet.direction == Direction::Up {
                b.amount_won += reward;
                b.total_won += 1;
            } else {
                b.amount_lost += user_bet.amount;
                b.total_lost += 1;
            }

            Ok(b)
        })?;
    }

    // Update the current round id
    CONFIG.update(deps.storage, |mut a| -> StdResult<_> {
        a.current_round_id += 1;

        Ok(a)
    })?;

    Ok(Response::new()
        .add_attribute("action", "end_round")
        .add_attribute("round-id", config.current_round_id.to_string()))
}

fn execute_pause(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // Check if user is admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // pasue the contract
    config.paused = true;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "pause"))
}
fn execute_resume(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // check if user is admin
    if info.sender != config.admin {
        return (Err(ContractError::Unauthorized {}));
    }

    // resume the contract
    config.paused = false;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "resume"))
}
fn execute_withdraw(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: String,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // Validate admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let bank_msg = BankMsg::Send {
        to_address: recipient,
        amount: vec![Coin {
            denom: "uzig".to_string(),
            amount,
        }],
    };

    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("amount", amount)
        .add_attribute("recipient", info.sender)
        .add_message(bank_msg))
}

// USER EXECUTE FUNCTION
fn execute_place_bet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    round_id: u64,
    direction: Direction,
) -> Result<Response, ContractError> {
    // Acess the storage
    let config = CONFIG.load(deps.storage)?;
    let mut round = ROUNDS.load(deps.storage, round_id)?;

    // Chekck if the contract is paused
    if config.paused {
        return Err(ContractError::ContractPaused {});
    }

    // Check if the current round is active
    let current_time = env.block.time;
    if current_time < round.start_time {
        return Err(ContractError::RoundNotStarted {});
    }
    if current_time >= round.lock_time {
        return Err(ContractError::RoundLocked {});
    }

    // check if user leaderboard exists
    if LEADERBOARDENTRY
        .may_load(deps.storage, &info.sender)?
        .is_none()
    {
        let board: LeaderboardEntry = LeaderboardEntry {
            user: info.sender.clone(),
            total_amount_played: Uint128::zero(),
            total_won: 0,
            total_lost: 0,
            amount_lost: Uint128::zero(),
            amount_won: Uint128::zero(),
            total_up: 0,
            total_down: 0,
        };
        LEADERBOARDENTRY.save(deps.storage, &info.sender, &board)?;
    }

    // Check if the user already placed a bet
    let user_address = info.sender.clone();
    let user_bet_key = (round_id, &user_address);
    if USER_BETS.may_load(deps.storage, user_bet_key)?.is_some() {
        return Err(ContractError::BetAlreadyPlaced {});
    }

    // Check if user sent token
    let bet_amount = info
        .funds
        .iter()
        .find(|c| c.denom == "uzig")
        .map(|c| c.amount)
        .unwrap_or_else(Uint128::zero);

    if bet_amount.is_zero() {
        return Err(ContractError::NoFundSent {});
    }

    // Save the user bet details
    let user_bet = Bet {
        amount: bet_amount,
        direction: direction.clone(),
    };
    USER_BETS.save(deps.storage, (round_id, &user_address), &user_bet)?;

    // Update the pool
    match direction {
        Direction::Up => {
            round.bull_pool += bet_amount;
        }
        Direction::Down => {
            round.bear_pool += bet_amount;
        }
    }

    round.total_pool += bet_amount;
    round.participants.push(user_address);

    // leaderboard entry
    let mut board: LeaderboardEntry = LEADERBOARDENTRY.load(deps.storage, &info.sender)?;
    board.total_amount_played += bet_amount;
    match direction {
        Direction::Down => {
            board.total_down += 1;
        }
        Direction::Up => {
            board.total_up += 1;
        }
    }

    // Save round
    ROUNDS.save(deps.storage, round_id, &round)?;

    Ok(Response::new()
        .add_attribute("action", "place_bet")
        .add_attribute("round_id", round_id.to_string())
        .add_attribute("user", info.sender.to_string())
        .add_attribute("amount", bet_amount)
        .add_attribute(
            ("direction"),
            match direction {
                Direction::Down => "Down",
                Direction::Up => "Up",
            },
        ))
}
fn execute_claim_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    round_id: u64,
) -> Result<Response, ContractError> {
    let round: Round = ROUNDS.load(deps.storage, round_id)?;
    let config: Config = CONFIG.load(deps.storage)?;

    // check if round is still active
    if !round.executed {
        return Err(ContractError::RoundNotEnded {});
    }

    let bet = USER_BETS
        .may_load(deps.storage, (round_id, &info.sender))?
        .ok_or(ContractError::BetNotFound {})?;

    // get user bet and sned reward
    let winning_pool: Uint128 = match round.result {
        Some(Direction::Up) => round.bull_pool,
        Some(Direction::Down) => round.bear_pool,
        None => return Err(ContractError::RoundNotEnded {}),
    };
    let losing_pool: Uint128 = round.total_pool - winning_pool;
    let reward: Uint128 = (bet.amount * losing_pool) / winning_pool + bet.amount;

    // send feee to treasury
    let percentage: u64 = config.treasury_fee / 100;
    let fee: Uint128 = reward * Uint128::new(percentage.into());
    let t_fee: Uint128 = reward - fee;

    let treasury_fee = BankMsg::Send {
        to_address: config.admin.to_string(),
        amount: vec![Coin {
            denom: "uzig".to_string(),

            amount: t_fee,
        }],
    };

    // Send the user reward
    let user_reward = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: "uzig".to_string(),
            amount: reward,
        }],
    };

    Ok(Response::new()
        .add_attribute("action", "claim_reward")
        .add_attribute("round_id", round_id.to_string())
        .add_attribute("user", info.sender.to_string())
        .add_attribute("reward", reward)
        .add_message(user_reward)
        .add_message(treasury_fee))
}
