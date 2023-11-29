use concordium_std::*;

use crate::{
  error::{ContractError, ContractResult},
  state::State,
};

#[derive(Debug, Serialize, SchemaType)]
pub struct SetMinter {
  pub minter: AccountAddress,
}

#[receive(
  contract = "test_nft",
  name = "setMinter",
  parameter = "SetMinter",
  error = "ContractError",
  mutable
)]
fn contract_set_minter(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
  ensure!(
    ctx.sender().matches_account(&ctx.owner()),
    ContractError::Unauthorized
  );

  let params: SetMinter = ctx.parameter_cursor().get()?;
  host.state_mut().set_minter(params.minter);
  Ok(())
}
