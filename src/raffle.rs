
use core::ops::{Add, Sub, Mul, Div};

use alloc::{
    string::{String, ToString},
    vec::Vec,
    vec,
    boxed::Box
};

use crate::{
    error::Error,
    interfaces::cep18::CEP18,
    utils::{get_current_address, get_key, self , read_from}, events::VestingEvent,
    events::emit, enums::Address
};

use casper_types::{
     account::AccountHash, U256,EntryPoint, Key, 
    ContractHash,EntryPointAccess,CLType,Parameter,EntryPointType,EntryPoints,
    contracts::NamedKeys,U512,RuntimeArgs,runtime_args, URef,CLValue
};
use casper_types_derive::{CLTyped, FromBytes, ToBytes};

use casper_contract::contract_api::{runtime, storage,system,account};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;

// variables
const NAME: &str = "name";
const START_DATE: &str = "start_date";
const END_DATE: &str = "end_date";
const COLLECTION: &str = "collection";
const NFT_INDEX: &str = "nft_index";
const PRICE: &str = "price";
const OWNER: &str = "owner";
const PURSE: &str = "purse";
const PARTIPICANT_COUNT : &str = "partipiciant_count";
const PARTIPICANT_DICT : &str = "partipiciant_dict";

//entry points
const ENTRY_POINT_DRAW: &str = "draw";
const ENTRY_POINT_CLAIM: &str = "claim";
const ENTRY_POINT_DEPOSIT: &str = "deposit";
const ENTRY_POINT_GET_PRICE: &str = "get_price";
const ENTRY_POINT_GET_PURSE: &str = "get_purse";

#[no_mangle]
pub extern "C" fn draw() {}

#[no_mangle]
pub extern "C" fn get_price() {
  let price : U512 = utils::read_from(PRICE);

  runtime::ret(CLValue::from_t(price.clone()).unwrap());
}

#[no_mangle]
pub extern "C" fn get_purse() {
  let raffle_purse = match runtime::get_key(PURSE) {
      Some(purse_key) => purse_key.into_uref().unwrap_or_revert(),
      None => {
          let new_purse = system::create_purse();
          runtime::put_key(PURSE, new_purse.into());
          new_purse
      }
  };  

  runtime::ret(CLValue::from_t(raffle_purse.into_add()).unwrap_or_revert());
}


#[no_mangle]
pub extern "C" fn claim() {
  let token_id  : u64 = utils::read_from(NFT_INDEX);
  let collection : Key = utils::read_from(COLLECTION);
  let caller: AccountHash = runtime::get_caller();

  let contract_address = get_current_address();

  let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

  // change to account with random account hash
  transfer(collection_hash,contract_address.into(), caller.into(), token_id);
}

// admin function
#[no_mangle]
pub extern "C" fn deposit() {
  let contract_address = get_current_address();
  let caller: AccountHash  = runtime::get_caller();
  let token_id : u64 = utils::read_from(NFT_INDEX);
  let collection : Key = utils::read_from(COLLECTION);

  let collection_hash: ContractHash = collection
  .into_hash()
  .map(ContractHash::new)
  .unwrap();

  // check owner is caller
  transfer(collection_hash, caller.into(),contract_address.into(), token_id);

  storage::new_dictionary(PARTIPICANT_DICT).unwrap_or_default();

  runtime::put_key(PARTIPICANT_COUNT, storage::new_uref(0u64).into());
}

// 
#[no_mangle]
pub extern "C" fn call() {
  let name: String = runtime::get_named_arg(NAME);
  let start_date: u64 = runtime::get_named_arg(START_DATE);
  let end_date: u64 = runtime::get_named_arg(END_DATE);
  let nft_index: u64 = runtime::get_named_arg(NFT_INDEX);
  let price: U512 = runtime::get_named_arg(PRICE);
  let collection: Key = runtime::get_named_arg(COLLECTION);
  //utils
  let owner : AccountHash = runtime::get_caller().into();
  
  let mut named_keys = NamedKeys::new();

  named_keys.insert(NAME.to_string(), storage::new_uref(name.clone()).into());
  named_keys.insert(START_DATE.to_string(), storage::new_uref(start_date).into());
  named_keys.insert(END_DATE.to_string(), storage::new_uref(end_date).into());
  named_keys.insert(PRICE.to_string(), storage::new_uref(price).into());
  named_keys.insert(OWNER.to_string(), storage::new_uref(owner).into());
  named_keys.insert(COLLECTION.to_string(), storage::new_uref(collection).into());
  named_keys.insert(NFT_INDEX.to_string(), storage::new_uref(nft_index).into());

 let draw_entry_point = EntryPoint::new(
    ENTRY_POINT_DRAW,
    vec![],
    CLType::URef,
    EntryPointAccess::Public,
    EntryPointType::Contract,
 );  

 let claim_entry_point = EntryPoint::new(
    ENTRY_POINT_CLAIM,
    vec![],
    CLType::URef,
    EntryPointAccess::Public,
    EntryPointType::Contract,
 );  

 let deposit_entry_point = EntryPoint::new(
  ENTRY_POINT_DEPOSIT,
  vec![],
  CLType::URef,
  EntryPointAccess::Public,
  EntryPointType::Contract,
  );

  let get_price_entry_point = EntryPoint::new(
  ENTRY_POINT_GET_PRICE,
  vec![],
  CLType::U512,
  EntryPointAccess::Public,
  EntryPointType::Contract,
  ); 

  let get_purse_entry_point = EntryPoint::new(
  ENTRY_POINT_GET_PURSE,
  vec![],
   CLType::URef,
  EntryPointAccess::Public,
  EntryPointType::Contract,
  );

  let now : u64 = runtime::get_blocktime().into();

  let mut entry_points = EntryPoints::new();
  entry_points.add_entry_point(draw_entry_point);
  entry_points.add_entry_point(claim_entry_point);
  entry_points.add_entry_point(deposit_entry_point);
  entry_points.add_entry_point(get_price_entry_point);
  entry_points.add_entry_point(get_purse_entry_point);

  let str1 = name.clone() + "_" + &now.to_string();
  
  let str2 = String::from("raffles_package_hash_");
  let str3 = String::from("raffles_access_uref_");
  let str4 = String::from("raffles_contract_hash_");
  let hash_name = str2 + &str1;
  let uref_name = str3 + &str1;
  let contract_hash_text = str4 + &str1;
 
   let (contract_hash, _contract_version) = storage::new_contract(
         entry_points,
         Some(named_keys),
         Some(hash_name.to_string()),
         Some(uref_name.to_string()),
    );

  runtime::put_key(&contract_hash_text.to_string(), contract_hash.into());
}


pub fn get_approved(contract_hash: ContractHash, owner: Address, token_id: u64) -> Option<Key> {
  runtime::call_contract::<Option<Key>>(
      contract_hash,
      "get_approved",
      runtime_args! {
        "owner" => owner,
        "token_id" => token_id
      },
  )
}

pub fn transfer(contract_hash: ContractHash , sender: Key, recipient: Key, token_id: u64) -> () {
  runtime::call_contract::<()>(
      contract_hash,
      "transfer",
      runtime_args! {
          "token_id" => token_id,
          "source_key" => sender,
          "target_key" => recipient,
      },
  )
}