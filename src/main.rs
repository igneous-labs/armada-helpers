use std::{str::FromStr, sync::Arc};

use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

use crate::clmm_vault::load_token_a_token_b_aum;

mod clmm_vault;
mod whirlpool;

anchor_lang::declare_id!("ArmN3Av2boBg8pkkeCK9UuCN9zSUVc2UQg1qR2sKwm8d");

#[tokio::main]
async fn main() {
    let client = Arc::new(RpcClient::new_with_commitment(String::from("***REMOVED***"), CommitmentConfig::confirmed()));
    println!("Hello, world!");

    // Load the CLMM Vault
    let clmm_vault = crate::clmm_vault::get_clmm_vault(client.clone(), Pubkey::from_str("7c75jrcMMJVEPtr1hwdBQTMJCKpjquGQk9b3p237vYyc").unwrap()).await;
    println!("mintA: {} mintB: {}", clmm_vault.token_mint_a, clmm_vault.token_mint_b);
    load_token_a_token_b_aum(client.clone(), &clmm_vault).await;
    // TODO: Load the LP mint data
    // TODO: Load all LP token hodlers and their LP balances
}
