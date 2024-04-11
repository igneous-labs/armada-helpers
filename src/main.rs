use std::{str::FromStr, sync::Arc};

use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

mod clmm_vault;

#[tokio::main]
async fn main() {
    let client = Arc::new(RpcClient::new_with_commitment(String::from("***REMOVED***"), CommitmentConfig::confirmed()));
    println!("Hello, world!");

    // TODO: Load the CLMM Vault
    let clmm_vault = crate::clmm_vault::get_clmm_vault(client, Pubkey::from_str("7c75jrcMMJVEPtr1hwdBQTMJCKpjquGQk9b3p237vYyc").unwrap()).await;
    println!("mintA: {} mintB: {}", clmm_vault.token_mint_a, clmm_vault.token_mint_b);
    // TODO: Load the CLMM Vault's token accounts
    // TODO: Load the LP mint data
    // TODO: Load all LP token hodlers and their LP balances
    // TODO: Load CLMM whirlpool
    // TODO: Load CLMM whirlpool Positions
    // TODO: Calculate the tokenA and tokenB value of the positions
}
