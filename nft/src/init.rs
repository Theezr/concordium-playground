use concordium_std::*;

use crate::state::State;

#[derive(Serialize, SchemaType, Debug)]
pub struct InitParams {
  pub minter: AccountAddress,
  pub mint_start: u64,    // Unix milliseconds
  pub mint_deadline: u64, // Unix milliseconds
  pub max_total_supply: u32,
}

/// Initialize contract instance with no token types initially.
#[init(
  contract = "test_nft",
  parameter = "InitParams",
  event = "Cis2Event<ContractTokenId, ContractTokenAmount>"
)]
fn contract_init(ctx: &InitContext, state_builder: &mut StateBuilder) -> InitResult<State> {
  let params: InitParams = ctx.parameter_cursor().get()?;
  // Construct the initial contract state.
  Ok(State::init(state_builder, params))
}
