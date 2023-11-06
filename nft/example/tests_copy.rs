//! Tests for the `test_nft` contract.
use concordium_cis2::*;
use concordium_smart_contract_testing::*;
use concordium_std::{collections::BTreeSet, concordium_test};
use test_nft::*;

/// The tests accounts.
const OWNER: AccountAddress = AccountAddress([0; 32]);
const OWNER_ADDR: Address = Address::Account(OWNER);
const MINTER: AccountAddress = AccountAddress([1; 32]);
const MINTER_ADDR: Address = Address::Account(MINTER);
const USER: AccountAddress = AccountAddress([1; 32]);
const USER_ADDR: Address = Address::Account(MINTER);

/// Token IDs.
const TOKEN_0: ContractTokenId = TokenIdU32(2);
const TOKEN_1: ContractTokenId = TokenIdU32(42);

/// Initial balance of the accounts.
const ACC_INITIAL_BALANCE: Amount = Amount::from_ccd(10000);

/// A signer for all the transactions.
const SIGNER: Signer = Signer::with_one_key();

/// Test minting succeeds and the tokens are owned by the given address and
/// the appropriate events are logged.
#[concordium_test]
fn test_minting() {
  let (chain, contract_address, update) = initialize_contract_and_mint_to_owner();

  // Invoke the view entrypoint and check that the tokens are owned by Alice.
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.view".to_string()),
        address: contract_address,
        message: OwnedParameter::empty(),
      },
    )
    .expect("Invoke view");

  // Check that the tokens are owned by Alice.
  let rv: ViewState = invoke.parse_return_value().expect("ViewState return value");
  println!("rv: {:?}", rv);

  assert_eq!(rv.all_tokens[..], [TOKEN_0, TOKEN_1]);
  assert_eq!(
    rv.state,
    vec![(
      OWNER_ADDR,
      ViewAddressState {
        owned_tokens: vec![TOKEN_0, TOKEN_1],
        operators: Vec::new(),
      }
    )]
  );

  // Check that the events are logged.
  let events = update.events().flat_map(|(_addr, events)| events);

  let events: Vec<Cis2Event<ContractTokenId, ContractTokenAmount>> = events
    .map(|e| e.parse().expect("Deserialize event"))
    .collect();

  assert_eq!(
    events,
    [
      Cis2Event::Mint(MintEvent {
        token_id: TokenIdU32(2),
        amount: TokenAmountU8(1),
        owner: OWNER_ADDR,
      }),
      Cis2Event::TokenMetadata(TokenMetadataEvent {
        token_id: TokenIdU32(2),
        metadata_url: MetadataUrl {
          url: format!("{TOKEN_METADATA_BASE_URL}02000000"),
          hash: None,
        },
      }),
      Cis2Event::Mint(MintEvent {
        token_id: TokenIdU32(42),
        amount: TokenAmountU8(1),
        owner: OWNER_ADDR,
      }),
      Cis2Event::TokenMetadata(TokenMetadataEvent {
        token_id: TokenIdU32(42),
        metadata_url: MetadataUrl {
          url: format!("{TOKEN_METADATA_BASE_URL}2A000000"),
          hash: None,
        },
      }),
    ]
  );
}

/// Test regular transfer where sender is the owner.
#[concordium_test]
fn test_account_transfer() {
  let (mut chain, contract_address, _update) = initialize_contract_and_mint_to_owner();

  // Transfer `TOKEN_0` from Alice to Bob.
  let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
    from: OWNER_ADDR,
    to: Receiver::Account(MINTER),
    token_id: TOKEN_0,
    amount: TokenAmountU8(1),
    data: AdditionalData::empty(),
  }]);

  let update = chain
    .contract_update(
      SIGNER,
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.transfer".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
      },
    )
    .expect("Transfer tokens");

  // Check that Bob now has `TOKEN_0` and that Alice still has `TOKEN_1`.
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
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
        OWNER_ADDR,
        ViewAddressState {
          owned_tokens: vec![TOKEN_1],
          operators: Vec::new(),
        }
      ),
      (
        MINTER_ADDR,
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
      from: OWNER_ADDR,
      to: MINTER_ADDR,
    }),]
  );
}

/// Test that you can add an operator.
/// Initialize the contract with two tokens owned by Alice.
/// Then add Bob as an operator for Alice.
#[concordium_test]
fn test_add_operator() {
  let (mut chain, contract_address, _update) = initialize_contract_and_mint_to_owner();

  // Add Bob as an operator for Alice.
  let params = UpdateOperatorParams(vec![UpdateOperator {
    update: OperatorUpdate::Add,
    operator: MINTER_ADDR,
  }]);

  let update = chain
    .contract_update(
      SIGNER,
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.updateOperator".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
      },
    )
    .expect("Update operator");

  // Check that an operator event occurred.
  let events = update
    .events()
    .flat_map(|(_addr, events)| events.iter().map(|e| e.parse().expect("Deserialize event")))
    .collect::<Vec<Cis2Event<ContractTokenId, ContractTokenAmount>>>();
  assert_eq!(
    events,
    [Cis2Event::UpdateOperator(UpdateOperatorEvent {
      operator: MINTER_ADDR,
      owner: OWNER_ADDR,
      update: OperatorUpdate::Add,
    }),]
  );

  // Construct a query parameter to check whether Bob is an operator for Alice.
  let query_params = OperatorOfQueryParams {
    queries: vec![OperatorOfQuery {
      owner: OWNER_ADDR,
      address: MINTER_ADDR,
    }],
  };

  // Invoke the operatorOf view entrypoint and check that Bob is an operator for
  // Alice.
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.operatorOf".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&query_params).expect("OperatorOf params"),
      },
    )
    .expect("Invoke view");

  let rv: OperatorOfQueryResponse = invoke
    .parse_return_value()
    .expect("OperatorOf return value");
  assert_eq!(rv, OperatorOfQueryResponse(vec![true]));
}

/// Test that a transfer fails when the sender is neither an operator or the
/// owner. In particular, Bob will attempt to transfer one of Alice's tokens to
/// himself.
#[concordium_test]
fn test_unauthorized_sender() {
  let (mut chain, contract_address, _update) = initialize_contract_and_mint_to_owner();

  // Construct a transfer of `TOKEN_0` from Alice to Bob, which will be submitted
  // by Bob.
  let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
    from: OWNER_ADDR,
    to: Receiver::Account(MINTER),
    token_id: TOKEN_0,
    amount: TokenAmountU8(1),
    data: AdditionalData::empty(),
  }]);

  // Notice that Bob is the sender/invoker.
  let update = chain
    .contract_update(
      SIGNER,
      MINTER,
      MINTER_ADDR,
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

/// Test that an operator can make a transfer.
#[concordium_test]
fn test_operator_can_transfer() {
  let (mut chain, contract_address, _update) = initialize_contract_and_mint_to_owner();

  // Add Bob as an operator for Alice.
  let params = UpdateOperatorParams(vec![UpdateOperator {
    update: OperatorUpdate::Add,
    operator: MINTER_ADDR,
  }]);
  chain
    .contract_update(
      SIGNER,
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.updateOperator".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&params).expect("UpdateOperator params"),
      },
    )
    .expect("Update operator");

  // Let Bob make a transfer to himself on behalf of Alice.
  let transfer_params = TransferParams::from(vec![concordium_cis2::Transfer {
    from: OWNER_ADDR,
    to: Receiver::Account(MINTER),
    token_id: TOKEN_0,
    amount: TokenAmountU8(1),
    data: AdditionalData::empty(),
  }]);

  chain
    .contract_update(
      SIGNER,
      MINTER,
      MINTER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.transfer".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&transfer_params).expect("Transfer params"),
      },
    )
    .expect("Transfer tokens");

  // Check that Bob now has `TOKEN_0` and that Alice still has `TOKEN_1`.
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
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
        OWNER_ADDR,
        ViewAddressState {
          owned_tokens: vec![TOKEN_1],
          operators: vec![MINTER_ADDR],
        }
      ),
      (
        MINTER_ADDR,
        ViewAddressState {
          owned_tokens: vec![TOKEN_0],
          operators: Vec::new(),
        }
      ),
    ]
  );
}

/// Helper function that sets up the contract with two tokens minted to
/// Alice, `TOKEN_0` and `TOKEN_1`.
fn initialize_contract_and_mint_to_owner() -> (Chain, ContractAddress, ContractInvokeSuccess) {
  let (mut chain, contract_address) = initialize_chain_and_contract();

  let mint_params = MintParams {
    owner: OWNER_ADDR,
    tokens: BTreeSet::from_iter(vec![TOKEN_0, TOKEN_1]),
  };

  // Mint two tokens to Alice.
  let update = chain
    .contract_update(
      SIGNER,
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.mint".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&mint_params).expect("Mint params"),
      },
    )
    .expect("Mint tokens");

  (chain, contract_address, update)
}

/// Setup chain and contract.
///
/// Also creates the two accounts, Alice and Bob.
///
/// Alice is the owner of the contract.
fn initialize_chain_and_contract() -> (Chain, ContractAddress) {
  let mut chain = Chain::new();

  // Create some accounts accounts on the chain.
  chain.create_account(Account::new(OWNER, ACC_INITIAL_BALANCE));
  chain.create_account(Account::new(MINTER, ACC_INITIAL_BALANCE));

  // Load and deploy the module.
  let module = module_load_v1("nft_test.wasm.v1").expect("Module exists");
  let deployment = chain
    .module_deploy_v1(SIGNER, OWNER, module)
    .expect("Deploy valid module");

  // Initialize the auction contract.
  let init = chain
    .contract_init(
      SIGNER,
      OWNER,
      Energy::from(10000),
      InitContractPayload {
        amount: Amount::zero(),
        mod_ref: deployment.module_reference,
        init_name: OwnedContractName::new_unchecked("init_test_nft".to_string()),
        param: OwnedParameter::empty(),
      },
    )
    .expect("Initialize contract");

  (chain, init.contract_address)
}
