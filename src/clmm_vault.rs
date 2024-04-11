use std::sync::Arc;

use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
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

pub async fn load_token_a_token_b_aum(client: Arc<RpcClient>, clmm: &ClpVault) {
  let keys = [clmm.clp, clmm.token_vault_a, clmm.token_vault_b];
  // Load the CLMM Vault's token accounts and whirlpool
  let res = client.get_multiple_accounts(&keys).await.expect("Loaded CLP and token vaults OK");
  for (index, acct) in res.iter().enumerate() {
    if index == 0 {
      let mut data: &[u8] = &acct.as_ref().expect("Whirlpool exists").data;
      let clp: crate::whirlpool::Whirlpool = AccountDeserialize::try_deserialize(&mut data).expect("Whirlpool deserialized properly");
      println!("CLP {}", clp.token_mint_a);
    } else if index == 1 {
      let mut data: &[u8] = &acct.as_ref().expect("Token vault a exists").data;
      let token_vault_a: TokenAccount  = AccountDeserialize::try_deserialize(&mut data).expect("Token vault a deserialized properly");
      println!("Token Vault A: {}", token_vault_a.amount);
    } else if index == 2 {
      let mut data: &[u8] = &acct.as_ref().expect("Token vault b exists").data;
      let token_vault_a: TokenAccount  = AccountDeserialize::try_deserialize(&mut data).expect("Token vault a deserialized properly");
      println!("Token Vault B: {}", token_vault_a.amount);
    }
  }

  // TODO: Deserialize the token vaults and whirlpool.
  // TODO: Load CLMM whirlpool Positions
  // TODO: Calculate the tokenA and tokenB value of the positions
}
