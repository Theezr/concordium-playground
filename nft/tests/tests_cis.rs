//! Tests for the `test_nft` contract.
mod helpers;

use helpers::functions::*;
use helpers::init::*;

use concordium_cis2::*;
use concordium_smart_contract_testing::*;
use concordium_std::concordium_test;
use test_nft::error::ContractError;
use test_nft::{contract_view::*, mint::*};

/// Test regular transfer where sender is the owner.
#[concordium_test]
fn test_account_transfer() {
  let (mut chain, contract_address) = initialize_chain_and_contract(100);

  let mint_params = MintParams {
    owners: vec![USER_ADDR, USER_ADDR],
    tokens: vec![TOKEN_0, TOKEN_1],
    token_uris: vec!["ipfs://test".to_string(), "ipfs://test".to_string()],
  };

  mint_to_address(&mut chain, contract_address, mint_params, None, None).expect("Mint failed");

  // Transfer `TOKEN_0` from Alice to Bob.
  let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
    from: USER_ADDR,
    to: Receiver::Account(USER2),
    token_id: TOKEN_0,
    amount: TokenAmountU8(1),
    data: AdditionalData::empty(),
  }]);

  let update = chain
    .contract_update(
      SIGNER,
      USER,
      USER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.transfer".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
      },
    )
    .expect("Transfer tokens");

  // Check that User2 now has `TOKEN_0` and that User still has `TOKEN_1`.
  let rv: ViewState = get_view_state(&chain, contract_address);
  assert_eq!(
    rv.state,
    vec![
      (
        USER_ADDR,
        ViewAddressState {
          owned_tokens: vec![TOKEN_1],
          operators: Vec::new(),
        }
      ),
      (
        USER2_ADDR,
        ViewAddressState {
          owned_tokens: vec![TOKEN_0],
          operators: Vec::new(),
        }
      ),
    ]
  );

  // Check that the events are logged.
  let events = update
    .events()
    .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
    .collect::<Vec<Cis2Event<_, _>>>();

  assert_eq!(
    events,
    [Cis2Event::Transfer(TransferEvent {
      token_id: TOKEN_0,
      amount: TokenAmountU8(1),
      from: USER_ADDR,
      to: USER2_ADDR,
    }),]
  );
}

/// Test that an operator can make a transfer.
#[concordium_test]
fn test_operator_can_transfer() {
  let (mut chain, contract_address) = initialize_chain_and_contract(100);

  let mint_params = MintParams {
    owners: vec![USER_ADDR, USER_ADDR],
    tokens: vec![TOKEN_0, TOKEN_1],
    token_uris: vec!["ipfs://test".to_string(), "ipfs://test".to_string()],
  };

  mint_to_address(&mut chain, contract_address, mint_params, None, None).expect("Mint failed");
  // Add User2 as an operator for User.
  let params = UpdateOperatorParams(vec![UpdateOperator {
    update: OperatorUpdate::Add,
    operator: USER2_ADDR,
  }]);

  chain
    .contract_update(
      SIGNER,
      USER,
      USER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.updateOperator".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
      },
    )
    .expect("Update operator");

  // Let User2 make a transfer to himself on behalf of User.
  let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
    from: USER_ADDR,
    to: Receiver::Account(USER2),
    token_id: TOKEN_0,
    amount: TokenAmountU8(1),
    data: AdditionalData::empty(),
  }]);

  chain
    .contract_update(
      SIGNER,
      USER2,
      USER2_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.transfer".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
      },
    )
    .expect("Transfer tokens");

  // Check that User2 now has `TOKEN_0` and that User still has `TOKEN_1`.
  let invoke = chain
    .contract_invoke(
      USER,
      USER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.view".to_string()),
        address: contract_address,
        message: OwnedParameter::empty(),
      },
    )
    .expect("Invoke view");
  let rv: ViewState = invoke.parse_return_value().expect("ViewState return value");
  assert_eq!(
    rv.state,
    vec![
      (
        USER_ADDR,
        ViewAddressState {
          owned_tokens: vec![TOKEN_1],
          operators: vec![USER2_ADDR],
        }
      ),
      (
        USER2_ADDR,
        ViewAddressState {
          owned_tokens: vec![TOKEN_0],
          operators: Vec::new(),
        }
      ),
    ]
  );
}

/// Test that a transfer fails when the sender is neither an operator or the
/// owner. In particular, Bob will attempt to transfer one of Alice's tokens to
/// himself.
#[concordium_test]
fn test_unauthorized_sender() {
  let (mut chain, contract_address) = initialize_chain_and_contract(100);

  let mint_params = MintParams {
    owners: vec![USER_ADDR, USER_ADDR],
    tokens: vec![TOKEN_0, TOKEN_1],
    token_uris: vec!["ipfs://test".to_string(), "ipfs://test".to_string()],
  };

  mint_to_address(&mut chain, contract_address, mint_params, None, None).expect("Mint failed");

  // Construct a transfer of `TOKEN_0` from User to User3, which will be submitted
  // by USER3.
  let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
    from: USER_ADDR,
    to: Receiver::Account(USER3),
    token_id: TOKEN_0,
    amount: TokenAmountU8(1),
    data: AdditionalData::empty(),
  }]);

  // Notice that USER3 is the sender/invoker.
  let update = chain
    .contract_update(
      SIGNER,
      USER3,
      USER3_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.transfer".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
      },
    )
    .expect_err("Transfer tokens");

  // Check that the correct error is returned.
  let rv: ContractError = update
    .parse_return_value()
    .expect("ContractError return value");
  assert_eq!(rv, ContractError::Unauthorized);
}
