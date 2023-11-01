#![cfg_attr(not(feature = "std"), no_std)]

//! # A Concordium V1 smart contract
use concordium_std::*;
use core::{fmt::Debug, option};
use std::collections::BTreeMap;

type VotingOption = String;
type VotingIndex = u32;
type VotingCount = u32;

/// Your smart contract state.
#[derive(Serialize, SchemaType, Clone)]
pub struct State {
  description: String,
  options: Vec<VotingOption>,
  end_time: Timestamp,
  ballots: BTreeMap<AccountAddress, VotingIndex>,
}

#[derive(Serialize, SchemaType)]
struct InitParameter {
  description: String,
  options: Vec<VotingOption>,
  end_time: Timestamp,
}

/// Init function that creates a new smart contract.
#[init(contract = "voting", parameter = "InitParameter")]
fn init(ctx: &impl HasInitContext, _state_builder: &mut StateBuilder) -> InitResult<State> {
  let param: InitParameter = ctx.parameter_cursor().get()?;

  Ok(State {
    description: param.description,
    options: param.options,
    end_time: param.end_time,
    ballots: BTreeMap::new(),
  })
}

pub type MyInputType = bool;

/// Your smart contract errors.
#[derive(Debug, PartialEq, Eq, Reject, Serialize, SchemaType)]
pub enum ContractError {
  /// Failed parsing the parameter.
  #[from(ParseError)]
  ParseParams,
  VotingFinished,
  ContractVoter,
  InvalidVotingOption,
}

/// Receive function. The input parameter is the boolean variable `throw_error`.
///  If `throw_error == true`, the receive function will throw a custom error.
///  If `throw_error == false`, the receive function executes successfully.
#[receive(
  contract = "voting",
  name = "vote",
  parameter = "VotingOption",
  error = "ContractError",
  mutable
)]
fn vote(ctx: &ReceiveContext, host: &mut Host<State>) -> Result<(), ContractError> {
  if host.state().end_time < ctx.metadata().slot_time() {
    return Err(ContractError::VotingFinished);
  }
  let acc = match ctx.sender() {
    Address::Account(acc) => acc,
    Address::Contract(_) => return Err(ContractError::ContractVoter),
  };

  let voting_option: VotingOption = ctx.parameter_cursor().get()?;
  let voting_index = match host
    .state()
    .options
    .iter()
    .position(|option| *option == voting_option)
  {
    Some(index) => index as u32,
    None => return Err(ContractError::InvalidVotingOption),
  };

  host
    .state_mut()
    .ballots
    .entry(acc)
    .and_modify(|old_voting_index| *old_voting_index = voting_index)
    .or_insert(voting_index);

  Ok(())
}

#[derive(Serialize, SchemaType)]
struct VotingView {
  description: String,
  options: Vec<VotingOption>,
  end_time: Timestamp,
  tally: BTreeMap<VotingOption, VotingCount>,
}
/// View function that returns the content of the state.
#[receive(contract = "voting", name = "view", return_value = "VotingView")]
fn view<'b>(_ctx: &ReceiveContext, host: &'b Host<State>) -> ReceiveResult<VotingView> {
  let state = host.state();
  let description = state.description.clone();
  let options = state.options.clone();
  let end_time = state.end_time;
  let mut tally = BTreeMap::new();

  for (_, voting_index) in state.ballots.iter() {
    let voting_option = options[*voting_index as usize].clone();
    tally
      .entry(voting_option)
      .and_modify(|count| *count += 1)
      .or_insert(1);
  }
  Ok(VotingView {
    description,
    options,
    end_time,
    tally,
  })
}
