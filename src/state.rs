use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use crate::msg::{Direction, LeaderboardEntry};
use cosmwasm_schema::cw_serde;

/// Configuration of the contract
#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub paused: bool,
    pub treasury_fee: u64, // e.g., 5 means 5%
    pub current_round_id: u64
}

/// Round definition
#[cw_serde]
pub struct Round {
    pub id: u64,
    pub bull_pool: Uint128,
    pub bear_pool: Uint128,
    pub total_pool: Uint128,
    pub start_time: Timestamp,
    pub lock_time: Timestamp,
    pub end_time: Timestamp,
    pub start_price: Uint128,
    pub lock_price: Uint128,
    pub end_price: Uint128,
    pub result: Option<Direction>,
    pub executed: bool,
    pub participants: Vec<Addr>
}

/// Bet placed by a user
#[cw_serde]
pub struct Bet {
    pub amount: Uint128,
    pub direction: Direction,
}

pub const CONFIG: Item<Config> = Item::new("config");

// The current round ID
// pub const CURRENT_ROUND_ID: Item<u64> = Item::new("current_round_id");

/// Map round ID to round data
pub const ROUNDS: Map<u64, Round> = Map::new("rounds");

/// Map from (round_id, user_address) => Bet
pub const USER_BETS: Map<(u64, &Addr), Bet> = Map::new("user_bets");

/// User winnings claimable (used in ClaimReward)
pub const WINNINGS: Map<&Addr, Uint128> = Map::new("winnings");

/// Leaderboard stat (total won per user)
pub const LEADERBOARDENTRY: Map<&Addr, LeaderboardEntry> = Map::new("leaderboard");
