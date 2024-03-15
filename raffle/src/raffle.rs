use core::ops::Add;

use alloc::{
    string::{String, ToString},
    vec,
};

use crate::{
    enums::Address,
    error::Error,
    events::{emit, RaffleEvent},
    utils::{self, get_current_address, get_key},
};

use casper_types::{
    account::AccountHash, contracts::NamedKeys, runtime_args, CLType, CLValue, ContractHash,
    EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key, Parameter, RuntimeArgs, URef,
    U512,
};

use casper_contract::contract_api::{runtime, storage, system};
use casper_contract::unwrap_or_revert::UnwrapOrRevert;

use tiny_keccak::{Hasher, Sha3};

// variables
const NAME: &str = "name";
const START_DATE: &str = "start_date";
const END_DATE: &str = "end_date";
const COLLECTION: &str = "collection";
const NFT_INDEX: &str = "nft_index";
const PRICE: &str = "price";
const OWNER: &str = "owner";
const PURSE: &str = "purse";
const PARTIPICANT_COUNT: &str = "partipiciant_count";
const PARTIPICANT_DICT: &str = "partipiciant_dict";
const PARTIPICANT: &str = "partipiciant";
const WINNER: &str = "winner";
const STORAGE_KEY: &str = "storage_key";
const CLAIMED: &str = "claimed";

//entry points
const ENTRY_POINT_DRAW: &str = "draw";
const ENTRY_POINT_CLAIM: &str = "claim";
const ENTRY_POINT_DEPOSIT: &str = "deposit";
const ENTRY_POINT_GET_PRICE: &str = "get_price";
const ENTRY_POINT_GET_PURSE: &str = "get_purse";
const ENTRY_POINT_BUY_TICKET: &str = "buy_ticket";
const ENTRY_POINT_CANCEL: &str = "cancel";

#[no_mangle]
pub extern "C" fn cancel() {
    check_admin_account();

    let now: u64 = runtime::get_blocktime().into();
    let end_date: u64 = utils::read_from(END_DATE);

    if end_date.lt(&now) {
        runtime::revert(Error::TimeError);
    }

    let partipiciant_count: u64 = utils::read_from(PARTIPICANT_COUNT);
    let collection: Key = utils::read_from(COLLECTION);
    let token_id: u64 = utils::read_from(NFT_INDEX);

    if partipiciant_count > 0 {
        runtime::revert(Error::CancelError);
    }

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

    let caller: AccountHash = runtime::get_caller();
    let contract_address = get_current_address();

    transfer(
        collection_hash,
        contract_address.into(),
        caller.into(),
        token_id,
    );
    runtime::put_key(CLAIMED, storage::new_uref(true).into());
    runtime::put_key(END_DATE, storage::new_uref(now).into());
}

#[no_mangle]
pub extern "C" fn draw() {
    check_admin_account();

    let now: u64 = runtime::get_blocktime().into();
    let end_date: u64 = utils::read_from(END_DATE);

    //to-do @oguzhaniptes check replace winner

    if end_date.gt(&now) {
        runtime::revert(Error::TimeError);
    }

    // let partipiciant_dict: URef = *runtime::get_key(PARTIPICANT_DICT).unwrap().as_uref().unwrap();
    let partipiciant_count: u64 = utils::read_from(PARTIPICANT_COUNT);

    let input = now.to_string();
    let mut sha3 = Sha3::v256();

    sha3.update(input.as_ref());

    let mut hash_bytes = [0u8; 32]; // SHA-3-256 for 32 byte
    sha3.finalize(&mut hash_bytes);

    let hash_number = bytes_to_u64(&hash_bytes);

    let random_winner = hash_number % partipiciant_count;

    runtime::put_key(WINNER, storage::new_uref(random_winner).into());

    let key = runtime::get_key(PURSE).unwrap_or_revert();
    let contract_purse: URef = key.into_uref().unwrap_or_revert();

    // let owner = runtime::get_caller();
    let owner = utils::read_from(OWNER);
    let balance: U512 = system::get_purse_balance(contract_purse).unwrap_or_revert();

    system::transfer_from_purse_to_account(contract_purse, owner, balance, None).unwrap();

    emit(
        &(RaffleEvent::Draw {
            winner: random_winner,
        }),
    )
}

#[no_mangle]
pub extern "C" fn buy_ticket() {
    let now: u64 = runtime::get_blocktime().into();
    let end_date: u64 = utils::read_from(END_DATE);

    if now.gt(&end_date) {
        runtime::revert(Error::TimeError);
    }

    let start_date: u64 = utils::read_from(START_DATE);

    if start_date.gt(&now) {
        runtime::revert(Error::TimeError);
    }

    let partipiciant: Key = runtime::get_named_arg(PARTIPICANT);

    let partipiciant_count: u64 = utils::read_from(PARTIPICANT_COUNT);

    let partipiciant_dict = *runtime::get_key(PARTIPICANT_DICT)
        .unwrap()
        .as_uref()
        .unwrap();

    storage::dictionary_put(
        partipiciant_dict,
        &partipiciant_count.to_string(),
        partipiciant,
    );

    runtime::put_key(
        PARTIPICANT_COUNT,
        storage::new_uref(partipiciant_count.add(1u64)).into(),
    );

    emit(&(RaffleEvent::BuyTicket { partipiciant }))
}

#[no_mangle]
pub extern "C" fn get_price() {
    let price: U512 = utils::read_from(PRICE);

    runtime::ret(CLValue::from_t(price).unwrap_or_revert());
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
    let token_id: u64 = utils::read_from(NFT_INDEX);
    let collection: Key = utils::read_from(COLLECTION);
    let caller: AccountHash = runtime::get_caller();

    let contract_address = get_current_address();

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

    let partipiciant_dict = *runtime::get_key(PARTIPICANT_DICT)
        .unwrap()
        .as_uref()
        .unwrap();
    let winner: u64 = utils::read_from(WINNER);

    let winner_partipiciant: Key = storage::dictionary_get(partipiciant_dict, &winner.to_string())
        .unwrap()
        .unwrap_or_revert_with(Error::WinnerError);

    if winner_partipiciant != Key::Account(caller) {
        runtime::revert(Error::WinnerError);
    }

    transfer(
        collection_hash,
        contract_address.into(),
        winner_partipiciant,
        token_id,
    );

    runtime::put_key(CLAIMED, storage::new_uref(true).into());

    emit(
        &(RaffleEvent::Claim {
            winner_partipiciant,
            collection,
            token_id,
        }),
    );
}

// admin function
#[no_mangle]
pub extern "C" fn deposit() {
    check_admin_account();

    let contract_address = get_current_address();
    let caller: AccountHash = runtime::get_caller();
    let token_id: u64 = utils::read_from(NFT_INDEX);
    let collection: Key = utils::read_from(COLLECTION);

    let collection_hash: ContractHash = collection.into_hash().map(ContractHash::new).unwrap();

    get_approved(collection_hash, caller.into(), token_id)
        .unwrap_or_revert_with(Error::NotApproved);

    // check owner is caller
    transfer(
        collection_hash,
        caller.into(),
        contract_address.into(),
        token_id,
    );

    storage::new_dictionary(PARTIPICANT_DICT).unwrap_or_default();

    runtime::put_key(PARTIPICANT_COUNT, storage::new_uref(0u64).into());
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String = runtime::get_named_arg(NAME);
    let start_date: u64 = runtime::get_named_arg(START_DATE);
    let end_date: u64 = runtime::get_named_arg(END_DATE);
    let nft_index: u64 = runtime::get_named_arg(NFT_INDEX);
    let price: U512 = runtime::get_named_arg(PRICE);
    let collection: Key = runtime::get_named_arg(COLLECTION);
    let storage_key: ContractHash = runtime::get_named_arg(STORAGE_KEY);
    //utils
    let owner: AccountHash = runtime::get_caller();
    let now: u64 = runtime::get_blocktime().into();

    let mut named_keys = NamedKeys::new();

    named_keys.insert(NAME.to_string(), storage::new_uref(name.clone()).into());
    named_keys.insert(START_DATE.to_string(), storage::new_uref(start_date).into());
    named_keys.insert(END_DATE.to_string(), storage::new_uref(end_date).into());
    named_keys.insert(PRICE.to_string(), storage::new_uref(price).into());
    named_keys.insert(OWNER.to_string(), storage::new_uref(owner).into());
    named_keys.insert(COLLECTION.to_string(), storage::new_uref(collection).into());
    named_keys.insert(NFT_INDEX.to_string(), storage::new_uref(nft_index).into());
    named_keys.insert(
        STORAGE_KEY.to_string(),
        storage::new_uref(storage_key).into(),
    );

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

    let buy_ticket_entry_point = EntryPoint::new(
        ENTRY_POINT_BUY_TICKET,
        vec![Parameter::new(PARTIPICANT, CLType::Key)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let cancel_entry_point = EntryPoint::new(
        ENTRY_POINT_CANCEL,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );

    let mut entry_points = EntryPoints::new();
    entry_points.add_entry_point(draw_entry_point);
    entry_points.add_entry_point(claim_entry_point);
    entry_points.add_entry_point(deposit_entry_point);
    entry_points.add_entry_point(get_price_entry_point);
    entry_points.add_entry_point(get_purse_entry_point);
    entry_points.add_entry_point(buy_ticket_entry_point);
    entry_points.add_entry_point(cancel_entry_point);

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

    runtime::call_contract::<()>(
        storage_key,
        "insert",
        runtime_args! {
            "data" => contract_hash.to_string(),
        },
    );
}

fn bytes_to_u64(bytes: &[u8]) -> u64 {
    let mut result: u64 = 0;
    for i in 0..8 {
        result |= (bytes[i] as u64) << ((7 - i) * 8);
    }
    result
}

pub fn check_admin_account() {
    let admin: AccountHash = get_key(OWNER);
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::AdminError);
    }
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

pub fn transfer(contract_hash: ContractHash, sender: Key, recipient: Key, token_id: u64) -> () {
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
