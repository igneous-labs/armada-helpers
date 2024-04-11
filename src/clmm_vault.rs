use std::sync::Arc;

use anchor_lang::prelude::*;
use clmm_bindings::ClpVault;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub async fn get_clmm_vault(client: Arc<RpcClient>, clmm_address: Pubkey) -> ClpVault {
  // Load the account
  let clmm_vault_acct = client.get_account(&clmm_address).await.expect("CLMM Vault did not exist");
  // Deserialize the account data
  let mut data: &[u8] = &clmm_vault_acct.data;
  AccountDeserialize::try_deserialize(&mut data).expect("deserialized properly")
}
