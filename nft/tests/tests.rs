//! Tests for the `test_nft` contract.
use concordium_cis2::*;
use concordium_smart_contract_testing::*;
use concordium_std::{collections::BTreeSet, concordium_test};
use test_nft::*;

/// The tests accounts.
const OWNER: AccountAddress = AccountAddress([1; 32]);
const OWNER_ADDR: Address = Address::Account(OWNER);
const MINTER: AccountAddress = AccountAddress([2; 32]);
const MINTER_ADDR: Address = Address::Account(MINTER);
const USER: AccountAddress = AccountAddress([3; 32]);
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
  let (mut chain, contract_address) = initialize_chain_and_contract();
  let update = mint_to_owner(&mut chain, contract_address);

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

  assert_eq!(rv.all_tokens[..], [TOKEN_0]);
  assert_eq!(
    rv.state,
    vec![(
      OWNER_ADDR,
      ViewAddressState {
        owned_tokens: vec![TOKEN_0],
        operators: Vec::new(),
      }
    )]
  );
  assert_eq!(rv.mint_count, vec![(TokenIdU32(2), 1)]);
  assert_eq!(rv.counter, 1);

  // Check that the events are logged.
  let events = update.events().flat_map(|(_addr, events)| events);
  let events: Vec<Cis2Event<ContractTokenId, ContractTokenAmount>> = events
    .map(|e| e.parse().expect("Deserialize event"))
    .collect();

  // println!("events: {:?}", events);

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
          url: "ipfs://test".to_string(),
          hash: None,
        },
      }),
    ]
  );
}

#[concordium_test]
fn test_contract_token_metadata() {
  let (mut chain, contract_address) = initialize_chain_and_contract();
  let update = mint_to_owner(&mut chain, contract_address);

  let token_ids = ContractTokenMetadataQueryParams {
    queries: vec![TOKEN_0],
  };

  // Invoke the view entrypoint and check that the tokens are owned by Alice.
  let invoke = chain
    .contract_invoke(
      OWNER,
      OWNER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.tokenMetadata".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&token_ids).expect("tokenIds params"),
      },
    )
    .expect("Invoke view");

  // Check that the tokenUri is set correctly
  let rv: TokenMetadataQueryResponse = invoke.parse_return_value().expect("ViewState return value");
  let TokenMetadataQueryResponse(urls) = rv;
  println!("rv: {:?}", urls);

  assert_eq!(
    urls,
    vec![MetadataUrl {
      url: "ipfs://test".to_string(),
      hash: None,
    }]
  );
}

/// Helper function that sets up the contract with two tokens minted to
/// Alice, `TOKEN_0` and `TOKEN_1`.
fn mint_to_owner(chain: &mut Chain, contract_address: ContractAddress) -> ContractInvokeSuccess {
  let mint_params = MintParams {
    owner: OWNER_ADDR,
    token: TOKEN_0,
    token_uri: "ipfs://test".to_string(),
  };

  // Mint two tokens to Alice.
  let update = chain
    .contract_update(
      SIGNER,
      MINTER,
      MINTER_ADDR,
      Energy::from(10000),
      UpdateContractPayload {
        amount: Amount::zero(),
        receive_name: OwnedReceiveName::new_unchecked("test_nft.mint".to_string()),
        address: contract_address,
        message: OwnedParameter::from_serial(&mint_params).expect("Mint params"),
      },
    )
    .expect("Mint tokens");

  update
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

  let params = InitParams { minter: MINTER };

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
        param: OwnedParameter::from_serial(&params).expect("Init params"),
      },
    )
    .expect("Initialize contract");

  (chain, init.contract_address)
}
