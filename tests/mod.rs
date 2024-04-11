use std::str::FromStr;

use armada_helpers::clmm_vault::{get_clmm_vault, load_token_a_token_b_aum};
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

const RPC_URL: &str = "YOUR_RPC_URL";
const CLMM_VAULT_KEY: &str = "7c75jrcMMJVEPtr1hwdBQTMJCKpjquGQk9b3p237vYyc";

#[tokio::test]
async fn main() {
    let client =
        RpcClient::new_with_commitment(String::from(RPC_URL), CommitmentConfig::confirmed());

    // Load the CLMM Vault
    let clmm_vault =
        get_clmm_vault(&client, &Pubkey::from_str(CLMM_VAULT_KEY).unwrap())
            .await;
    println!(
        "mintA: {} mintB: {}",
        clmm_vault.token_mint_a, clmm_vault.token_mint_b
    );
    let res = load_token_a_token_b_aum(&client, &clmm_vault).await;
    println!("Vault balances: {:?}", res);
    // TODO: Load all LP token hodlers and their LP balances
}
