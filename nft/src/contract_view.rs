use concordium_std::*;

use crate::{cis2::ContractTokenId, state::State};

#[derive(Serialize, SchemaType, PartialEq, Eq, Debug)]
pub struct ViewAddressState {
  pub owned_tokens: Vec<ContractTokenId>,
  pub operators: Vec<Address>,
}

#[derive(Serialize, SchemaType, PartialEq, Eq, Debug)]
pub struct ViewState {
  pub name: String,
  pub symbol: String,
  pub state: Vec<(Address, ViewAddressState)>,
  pub all_tokens: Vec<ContractTokenId>,
  pub token_uris: Vec<String>,
  pub counter: u32,
  pub mint_count: Vec<(ContractTokenId, u32)>,
  pub mint_start: u64,
  pub mint_deadline: u64,
  pub max_total_supply: u32,
}

/// View function that returns the entire contents of the state. Meant for
/// testing.
#[receive(contract = "test_nft", name = "view", return_value = "ViewState")]
fn contract_view(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<ViewState> {
  let state = host.state();

  let mut inner_state = Vec::new();
  for (k, a_state) in state.address_state.iter() {
    let owned_tokens = a_state.owned_tokens.iter().map(|x| *x).collect();
    let operators = a_state.operators.iter().map(|x| *x).collect();
    inner_state.push((
      *k,
      ViewAddressState {
        owned_tokens,
        operators,
      },
    ));
  }
  let all_tokens = state.all_tokens.iter().map(|x| *x).collect();
  let token_uris = state.token_uris.iter().map(|(_, v)| v.clone()).collect();
  let mint_count = state.mint_count.iter().map(|(k, v)| (*k, *v)).collect();

  Ok(ViewState {
    name: state.name.clone(),
    symbol: state.symbol.clone(),
    state: inner_state,
    all_tokens,
    token_uris,
    counter: state.counter,
    mint_count,
    mint_start: state.mint_start,
    mint_deadline: state.mint_deadline,
    max_total_supply: state.max_total_supply,
  })
}
