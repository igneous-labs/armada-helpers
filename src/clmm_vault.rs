use std::sync::Arc;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use clmm_bindings::{ClpVault, MAX_POSITIONS};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::whirlpool::{
    get_liquidity_from_position, get_token_a_b_reward_indexes, Position, Whirlpool,
};

pub async fn get_clmm_vault(client: Arc<RpcClient>, clmm_address: Pubkey) -> ClpVault {
    // Load the account
    let clmm_vault_acct = client
        .get_account(&clmm_address)
        .await
        .expect("CLMM Vault did not exist");
    // Deserialize the account data
    let mut data: &[u8] = &clmm_vault_acct.data;
    AccountDeserialize::try_deserialize(&mut data).expect("deserialized properly")
}

pub async fn load_token_a_token_b_aum(client: Arc<RpcClient>, clmm: &ClpVault) {
    let total_a = 0_u64;
    let total_b = 0_u64;
    let keys = [
        clmm.clp,
        clmm.token_vault_a,
        clmm.token_vault_b,
        clmm.positions[0].position_key,
        clmm.positions[1].position_key,
        clmm.positions[2].position_key,
        clmm.positions[3].position_key,
        clmm.positions[4].position_key,
    ];
    let mut positions: [Option<Position>; MAX_POSITIONS] = [None, None, None, None, None];
    // Load the CLMM Vault's token accounts and whirlpool
    let res = client
        .get_multiple_accounts(&keys)
        .await
        .expect("Loaded CLP and token vaults OK");
    for (index, acct) in res.iter().enumerate() {
        if index == 0 {
            let mut data: &[u8] = &acct.as_ref().expect("Whirlpool exists").data;
            let clp: crate::whirlpool::Whirlpool = AccountDeserialize::try_deserialize(&mut data)
                .expect("Whirlpool deserialized properly");
            println!("CLP {}", clp.token_mint_a);
        } else if index == 1 {
            let mut data: &[u8] = &acct.as_ref().expect("Token vault a exists").data;
            let token_vault_a: TokenAccount = AccountDeserialize::try_deserialize(&mut data)
                .expect("Token vault a deserialized properly");
            println!("Token Vault A: {}", token_vault_a.amount);
        } else if index == 2 {
            let mut data: &[u8] = &acct.as_ref().expect("Token vault b exists").data;
            let token_vault_a: TokenAccount = AccountDeserialize::try_deserialize(&mut data)
                .expect("Token vault a deserialized properly");
            println!("Token Vault B: {}", token_vault_a.amount);
        } else if index > 2 && index < 3 + MAX_POSITIONS {
          let position_index = index - 3;
          let position_key = clmm.positions[position_index].position_key;
          println!("position_key {}", position_key);
          if position_key == Pubkey::default() {
            continue;
          }
          let mut data: &[u8] = &acct.as_ref().expect(&format!("Position at idx {} exists", position_index)).data;
          let position: Position = AccountDeserialize::try_deserialize(&mut data).expect(
            &format!("Position at idx {} didn't deserialized properly", position_index)
          );
          println!("Position liquidity {}", position.liquidity);
          positions[position_index] = Some(position);
        }
    }

    // TODO: Deserialize the token vaults and whirlpool.
    // TODO: Calculate the tokenA and tokenB value of the positions
}

pub fn total_tokens_on_positions<'info>(
    whirlpool: &Whirlpool,
    clp_vault: &ClpVault,
    positions: &[Option<Position>; MAX_POSITIONS],
) -> Result<(u64, u64)> {
    let mut total_a: u64 = 0;
    let mut total_b: u64 = 0;
    let (token_a_reward_indexes, token_b_reward_indexes) =
        get_token_a_b_reward_indexes(&whirlpool, clp_vault.token_mint_a, clp_vault.token_mint_b);

    for (index, vault_position) in clp_vault.positions.iter().enumerate() {
        let position = &positions[index];
        // Skip if the position is empty
        if vault_position.position_key == Pubkey::default() && position.is_none() {
            continue;
        }
        let position = position.as_ref().unwrap();

        // get liquidity of the position
        let position_liquidity = position.liquidity;

        if position_liquidity > 0 {
            // Calculate the amount of tokenA and tokenB from Position liquidity
            let position_balances = get_liquidity_from_position(&position, &whirlpool);
            total_a += position_balances.a;
            total_b += position_balances.b;
        }

        let fees_owed_a = position.fee_owed_a;
        let fees_owed_b = position.fee_owed_b;
        total_a += fees_owed_a;
        total_b += fees_owed_b;

        // Loop through the Whirlpool rewards and accrue the uncollected amounts.
        total_a += token_a_reward_indexes
            .iter()
            .map(|&index| {
                let acc = position.reward_infos[index];
                acc.amount_owed
            })
            .sum::<u64>();

        total_b += token_b_reward_indexes
            .iter()
            .map(|&index| {
                let acc = position.reward_infos[index];
                acc.amount_owed
            })
            .sum::<u64>();
    }
    Ok((total_a, total_b))
}
