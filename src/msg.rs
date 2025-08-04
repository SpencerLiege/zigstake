use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub treasury_fee: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    PlaceBet { round_id: u64, direction: Direction},
    ClaimReward { round_id: u64},
    StartRound { price: Uint128 },
    LockRound {  price: Uint128 },
    EndRound {  price: Uint128 },
    Pause {},
    Resume {},
    Withdraw { amount: Uint128, recipient: String},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RoundResponse)]
    GetRoundDetails { round_id: u64 },

    #[returns(Vec<RoundResponse>)]
    GetAllRoundDetails {},

    #[returns(BetResponse)]
    GetUserPrediction { round_id: u64, user: String },

    #[returns(Vec<BetResponse>)]
    GetAllUserPredictions {},

    #[returns(bool)]
    IsPaused {},

    #[returns(Vec<LeaderboardEntry>)]
    GetLeaderboard {},

    #[returns(Uint128)]
    GetPool { round_id: u64 }
    
}

#[cw_serde]
// #[derive(Debug)]
pub enum Direction {
    Up,
    Down
}

#[cw_serde]
pub struct RoundResponse {
    pub id: u64,
    pub bull_pool: Uint128,
    pub bear_pool: Uint128,
    pub total_pool: Uint128,
    pub start_time: u64,
    pub lock_time: u64,
    pub end_time: u64,
    pub start_price: Uint128,
    pub lock_price: Uint128,
    pub end_price: Uint128,
    pub result: Option<Direction>,
    pub executed: bool,
    pub participants: Vec<Addr>
}

#[cw_serde]
pub struct BetResponse {
    pub amount: Uint128, 
    pub direction: Direction
}

#[cw_serde]
pub struct LeaderboardEntry {
    pub user: Addr,
    pub total_amount_played: Uint128, 
    pub total_won: u64,
    pub total_lost: u64,
    pub total_up: u64,
    pub total_down: u64,
    pub amount_won: Uint128,
    pub amount_lost: Uint128,

}