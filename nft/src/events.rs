use concordium_cis2::{MetadataUrl, MINT_EVENT_TAG, TOKEN_METADATA_EVENT_TAG, TRANSFER_EVENT_TAG};
use concordium_std::{collections::BTreeMap, schema::SchemaType, *};

use crate::cis2::{ContractTokenAmount, ContractTokenId, MintCountTokenID};

pub type TransferEvent = concordium_cis2::TransferEvent<ContractTokenId, ContractTokenAmount>;
pub type TokenMetadataEvent = concordium_cis2::TokenMetadataEvent<ContractTokenId>;
pub type MintEvent = concordium_cis2::MintEvent<ContractTokenId, ContractTokenAmount>;
pub type BurnEvent = concordium_cis2::BurnEvent<ContractTokenId, ContractTokenAmount>;

#[derive(Debug, Deserial, PartialEq, Eq, Serial, SchemaType)]
pub struct MintedEvent {
  pub token_id: ContractTokenId,
  pub mint_count: MintCountTokenID,
  pub timestamp: u64,
  pub token_uri: MetadataUrl,
}

#[derive(Debug, Deserial, PartialEq, Eq, Serial, SchemaType)]
pub struct DeployEvent {
  pub name: String,
  pub symbol: String,
  pub contract_uri: MetadataUrl,
  pub minter: AccountAddress,
  pub mint_start: u64,
  pub mint_deadline: u64,
  pub max_total_supply: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContractEvent {
  Mint(MintEvent),
  TokenMetadata(TokenMetadataEvent),
  Transfer(TransferEvent),
  Minted(MintedEvent),
  Deploy(DeployEvent),
}

const MINTED_EVENT_TAG: u8 = u8::MIN;
const DEPLOY_EVENT_TAG: u8 = u8::MIN + 1;

impl Serial for ContractEvent {
  fn serial<W: Write>(&self, out: &mut W) -> Result<(), W::Err> {
    match self {
      ContractEvent::Transfer(event) => {
        out.write_u8(concordium_cis2::TRANSFER_EVENT_TAG)?;
        event.serial(out)
      }
      ContractEvent::Mint(event) => {
        out.write_u8(concordium_cis2::MINT_EVENT_TAG)?;
        event.serial(out)
      }
      ContractEvent::TokenMetadata(event) => {
        out.write_u8(concordium_cis2::TOKEN_METADATA_EVENT_TAG)?;
        event.serial(out)
      }
      ContractEvent::Minted(event) => {
        out.write_u8(MINTED_EVENT_TAG)?;
        event.serial(out)
      }
      ContractEvent::Deploy(event) => {
        out.write_u8(DEPLOY_EVENT_TAG)?;
        event.serial(out)
      }
    }
  }
}

impl Deserial for ContractEvent {
  fn deserial<R: Read>(source: &mut R) -> ParseResult<Self> {
    // Read the tag from the source
    let tag = source.read_u8()?;

    // Match the tag to the correct variant
    match tag {
      TRANSFER_EVENT_TAG => {
        let event = TransferEvent::deserial(source)?;
        Ok(ContractEvent::Transfer(event))
      }
      MINT_EVENT_TAG => {
        let event = MintEvent::deserial(source)?;
        Ok(ContractEvent::Mint(event))
      }
      TOKEN_METADATA_EVENT_TAG => {
        let event = TokenMetadataEvent::deserial(source)?;
        Ok(ContractEvent::TokenMetadata(event))
      }
      MINTED_EVENT_TAG => {
        let event = MintedEvent::deserial(source)?;
        Ok(ContractEvent::Minted(event))
      }
      DEPLOY_EVENT_TAG => {
        let event = DeployEvent::deserial(source)?;
        Ok(ContractEvent::Deploy(event))
      }
      _ => Err(ParseError::default()),
    }
  }
}

impl SchemaType for ContractEvent {
  fn get_type() -> schema::Type {
    let mut event_map = BTreeMap::new();
    event_map.insert(
      TRANSFER_EVENT_TAG,
      (
        "Transfer".to_string(),
        schema::Fields::Named(vec![
          (String::from("token_id"), ContractTokenId::get_type()),
          (String::from("amount"), ContractTokenAmount::get_type()),
          (String::from("from"), Address::get_type()),
          (String::from("to"), Address::get_type()),
        ]),
      ),
    );
    event_map.insert(
      MINT_EVENT_TAG,
      (
        "Mint".to_string(),
        schema::Fields::Named(vec![
          (String::from("token_id"), ContractTokenId::get_type()),
          (String::from("amount"), ContractTokenAmount::get_type()),
          (String::from("owner"), Address::get_type()),
        ]),
      ),
    );
    event_map.insert(
      TOKEN_METADATA_EVENT_TAG,
      (
        "TokenMetadata".to_string(),
        schema::Fields::Named(vec![
          (String::from("token_id"), ContractTokenId::get_type()),
          (String::from("metadata_url"), MetadataUrl::get_type()),
        ]),
      ),
    );
    event_map.insert(
      MINTED_EVENT_TAG,
      (
        "Minted".to_string(),
        schema::Fields::Named(vec![
          (String::from("token_id"), ContractTokenId::get_type()),
          (String::from("mint_count"), MintCountTokenID::get_type()),
          (String::from("timestamp"), u64::get_type()),
          (String::from("token_uri"), MetadataUrl::get_type()),
        ]),
      ),
    );
    event_map.insert(
      DEPLOY_EVENT_TAG,
      (
        "Deploy".to_string(),
        schema::Fields::Named(vec![
          (String::from("name"), String::get_type()),
          (String::from("symbol"), String::get_type()),
          (String::from("contract_uri"), MetadataUrl::get_type()),
          (String::from("minter"), Address::get_type()),
          (String::from("mint_start"), u64::get_type()),
          (String::from("mint_deadline"), u64::get_type()),
          (String::from("max_total_supply"), u32::get_type()),
        ]),
      ),
    );
    schema::Type::TaggedEnum(event_map)
  }
}
