use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use clmm_bindings::{ClpVault, MAX_POSITIONS};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::whirlpool::{
    get_liquidity_from_position, get_token_a_b_reward_indexes, Position, Whirlpool,
};

pub async fn get_clmm_vault(client: &RpcClient, clmm_address: &Pubkey) -> ClpVault {
    // Load the account
    let clmm_vault_acct = client
        .get_account(clmm_address)
        .await
        .expect("CLMM Vault did not exist");
    // Deserialize the account data
    let mut data: &[u8] = &clmm_vault_acct.data;
    AccountDeserialize::try_deserialize(&mut data).expect("deserialized properly")
}

#[derive(Debug)]
pub struct ClmmBalances {
    /// The SPL Token Mint address of token A
    pub token_a: Pubkey,
    /// The SPL Token Mint address of token B
    pub token_b: Pubkey,
    /// The total amount of token A under management by the vault
    pub total_a: u64,
    /// The total amount of token B under management by the vault
    pub total_b: u64,
    /// The mint address for the LP token
    pub lp_mint: Pubkey,
    /// The total supply of the LP mint for the vault
    pub lp_supply: u64,
}

pub async fn load_token_a_token_b_aum(client: &RpcClient, clmm: &ClpVault) -> ClmmBalances {
    let mut total_a = 0_u64;
    let mut total_b = 0_u64;
    let mut lp_supply = 0_u64;
    let keys = [
        clmm.clp,
        clmm.token_vault_a,
        clmm.token_vault_b,
        clmm.lp_mint,
        clmm.positions[0].position_key,
        clmm.positions[1].position_key,
        clmm.positions[2].position_key,
        clmm.positions[3].position_key,
        clmm.positions[4].position_key,
    ];
    let mut positions: [Option<Position>; MAX_POSITIONS] = [None, None, None, None, None];
    let mut clp: Option<Whirlpool> = None;
    // Load the CLMM Vault's token accounts and whirlpool
    let res = client
        .get_multiple_accounts(&keys)
        .await
        .expect("Loaded CLP and token vaults OK");
    for (index, acct) in res.iter().enumerate() {
        if index == 0 {
            let mut data: &[u8] = &acct.as_ref().expect("Whirlpool exists").data;
            clp = Some(
                AccountDeserialize::try_deserialize(&mut data)
                    .expect("Whirlpool deserialized properly"),
            );
        } else if index == 1 {
            let mut data: &[u8] = &acct.as_ref().expect("Token vault a exists").data;
            let token_vault_a: TokenAccount = AccountDeserialize::try_deserialize(&mut data)
                .expect("Token vault a deserialized properly");
            total_a += token_vault_a.amount;
        } else if index == 2 {
            let mut data: &[u8] = &acct.as_ref().expect("Token vault b exists").data;
            let token_vault_b: TokenAccount = AccountDeserialize::try_deserialize(&mut data)
                .expect("Token vault a deserialized properly");
            total_b += token_vault_b.amount;
        } else if index == 3 {
            let mut data: &[u8] = &acct.as_ref().expect("LP Mint exists").data;
            let lp_mint_acct: Mint = AccountDeserialize::try_deserialize(&mut data)
                .expect("LP Mint deserialized properly");
            lp_supply += lp_mint_acct.supply;
        } else if index > 3 && index < 4 + MAX_POSITIONS {
            let position_index = index - 4;
            let position_key = clmm.positions[position_index].position_key;
            if position_key == Pubkey::default() {
                continue;
            }
            let mut data: &[u8] = &acct
                .as_ref()
                .expect(&format!("Position at idx {} exists", position_index))
                .data;
            let position: Position =
                AccountDeserialize::try_deserialize(&mut data).expect(&format!(
                    "Position at idx {} didn't deserialized properly",
                    position_index
                ));
            positions[position_index] = Some(position);
        }
    }
    // Calculate the tokenA and tokenB value of the positions
    let (positions_a, positions_b) = total_tokens_on_positions(&clp.unwrap(), clmm, &positions)
        .expect("Position balances calculated");
    println!("Position balances {:?}", (positions_a, positions_b));

    total_a += positions_a;
    total_b += positions_b;
    ClmmBalances {
        token_a: clmm.token_mint_a,
        token_b: clmm.token_mint_b,
        total_a,
        total_b,
        lp_mint: clmm.lp_mint,
        lp_supply,
    }
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
