use anchor_lang::prelude::*;
use whirlpool::math::{get_amount_delta_a, get_amount_delta_b, sqrt_price_from_tick_index};

// Number of rewards supported by Whirlpools
pub const NUM_REWARDS: usize = 3;

#[account]
#[derive(Default, Copy)]
#[repr(C)]
pub struct Whirlpool {
    pub whirlpools_config: Pubkey, // 32
    pub whirlpool_bump: [u8; 1],   // 1

    pub tick_spacing: u16,          // 2
    pub tick_spacing_seed: [u8; 2], // 2

    // Stored as hundredths of a basis point
    // u16::MAX corresponds to ~6.5%
    pub fee_rate: u16, // 2

    // Denominator for portion of fee rate taken (1/x)%
    pub protocol_fee_rate: u16, // 2

    // Maximum amount that can be held by Solana account
    pub liquidity: u128, // 16

    // MAX/MIN at Q32.64, but using Q64.64 for rounder bytes
    // Q64.64
    pub sqrt_price: u128,        // 16
    pub tick_current_index: i32, // 4

    pub protocol_fee_owed_a: u64, // 8
    pub protocol_fee_owed_b: u64, // 8

    pub token_mint_a: Pubkey,  // 32
    pub token_vault_a: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_a: u128, // 16

    pub token_mint_b: Pubkey,  // 32
    pub token_vault_b: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_b: u128, // 16

    pub reward_last_updated_timestamp: u64, // 8

    pub reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS], // 384
}
unsafe impl anchor_lang::__private::bytemuck::Pod for Whirlpool {}
unsafe impl anchor_lang::__private::bytemuck::Zeroable for Whirlpool {}

/// Stores the state relevant for tracking liquidity mining rewards at the `Whirlpool` level.
/// These values are used in conjunction with `PositionRewardInfo`, `Tick.reward_growths_outside`,
/// and `Whirlpool.reward_last_updated_timestamp` to determine how many rewards are earned by open
/// positions.
#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Default, Debug, PartialEq)]
pub struct WhirlpoolRewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that has permission to initialize the reward and set emissions.
    pub authority: Pubkey,
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub growth_global_x64: u128,
}

#[account]
#[derive(Default)]
pub struct Position {
    pub whirlpool: Pubkey,     // 32
    pub position_mint: Pubkey, // 32
    pub liquidity: u128,       // 16
    pub tick_lower_index: i32, // 4
    pub tick_upper_index: i32, // 4

    // Q64.64
    pub fee_growth_checkpoint_a: u128, // 16
    pub fee_owed_a: u64,               // 8
    // Q64.64
    pub fee_growth_checkpoint_b: u128, // 16
    pub fee_owed_b: u64,               // 8

    pub reward_infos: [PositionRewardInfo; NUM_REWARDS], // 72
}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Default, Debug, PartialEq)]
pub struct PositionRewardInfo {
    // Q64.64
    pub growth_inside_checkpoint: u128,
    pub amount_owed: u64,
}

pub fn get_token_a_b_reward_indexes(
  whirlpool: &Whirlpool,
  token_a_mint: Pubkey,
  token_b_mint: Pubkey,
) -> (Vec<usize>, Vec<usize>) {
  // Gather all the indexes for token a rewards and token b rewards. This is to avoid nested loops.
  let mut token_a_reward_indexes: Vec<usize> = Vec::with_capacity(NUM_REWARDS);
  let mut token_b_reward_indexes: Vec<usize> = Vec::with_capacity(NUM_REWARDS);
  // Search the whirlpool rewards for tokenA or tokenB
  for (reward_index, reward_info) in whirlpool.reward_infos.iter().enumerate() {
      if reward_info.mint == token_a_mint {
          token_a_reward_indexes.push(reward_index);
      }

      if reward_info.mint == token_b_mint {
          token_b_reward_indexes.push(reward_index);
      }
  }
  (token_a_reward_indexes, token_b_reward_indexes)
}


pub struct TokenBalances {
  pub a: u64,
  pub b: u64,
}

pub fn get_liquidity_from_position(position: &Position, whirlpool: &Whirlpool) -> TokenBalances {
  let sqrt_price_lower = sqrt_price_from_tick_index(position.tick_lower_index);
  let sqrt_price_upper = sqrt_price_from_tick_index(position.tick_upper_index);
  // bound out-or-range price (sqrt_price_lower <= sqrt_price_current <= sqrt_price_upper)
  let sqrt_price_current = std::cmp::min(
      std::cmp::max(whirlpool.sqrt_price, sqrt_price_lower),
      sqrt_price_upper,
  );

  let position_amount_a = get_amount_delta_a(
      sqrt_price_current,
      sqrt_price_upper,
      position.liquidity,
      false,
  )
  .unwrap();
  let position_amount_b = get_amount_delta_b(
      sqrt_price_lower,
      sqrt_price_current,
      position.liquidity,
      false,
  )
  .unwrap();

  TokenBalances {
      a: position_amount_a,
      b: position_amount_b,
  }
}