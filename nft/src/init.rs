use concordium_std::*;

use crate::{
  events::{ContractEvent, DeployEvent},
  state::State,
};

#[derive(Serialize, SchemaType, Debug)]
pub struct InitParams {
  pub name: String,
  pub symbol: String,
  pub contract_uri: MetadataUrl,
  pub minter: AccountAddress,
  pub mint_start: u64,    // Unix milliseconds
  pub mint_deadline: u64, // Unix milliseconds
  pub max_total_supply: u32,
}

/// Initialize contract instance with no token types initially.
#[init(
  contract = "test_nft",
  parameter = "InitParams",
  event = "ContractEvent",
  enable_logger
)]
fn contract_init(
  ctx: &InitContext,
  state_builder: &mut StateBuilder,
  logger: &mut Logger,
) -> InitResult<State> {
  let params: InitParams = ctx.parameter_cursor().get()?;

  logger.log(&ContractEvent::Deploy(DeployEvent {
    name: params.name.clone(),
    symbol: params.symbol.clone(),
    contract_uri: params.contract_uri.clone(),
    minter: params.minter,
    mint_start: params.mint_start,
    mint_deadline: params.mint_deadline,
    max_total_supply: params.max_total_supply,
  }))?;

  // Construct the initial contract state.
  Ok(State::init(state_builder, params))
}
