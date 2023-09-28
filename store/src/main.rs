#![no_main]

use std::ops::Add;

use casper_types::{
    EntryPoint,
    EntryPointAccess,
    CLType,
    Parameter,
    EntryPointType,
    EntryPoints,
    contracts::NamedKeys,
    bytesrepr::FromBytes,
    CLTyped,
    runtime_args,
    RuntimeArgs,
    account::AccountHash,
};

use casper_types::ApiError;

use casper_contract::unwrap_or_revert::UnwrapOrRevert;
use core::convert::TryInto;

use casper_contract::contract_api::{ runtime, storage };

const NAME: &str = "name";
const DATA_COUNT: &str = "data_count";
const DATA: &str = "data";
const OWNER: &str = "owner";

const DATA_DICT: &str = "data_dict";

const ENTRY_POINT_INSERT: &str = "insert";
const ENTRY_POINT_INIT: &str = "init";

#[no_mangle]
pub extern "C" fn insert() {
    let last_count: u64 = get_key(DATA_COUNT);
    let data: String = runtime::get_named_arg(DATA);

    let data_dict = *runtime::get_key(DATA_DICT).unwrap().as_uref().unwrap();

    storage::dictionary_put(data_dict, &last_count.to_string(), data);

    runtime::put_key(DATA_COUNT, storage::new_uref(last_count.add(1u64)).into());
}

#[no_mangle]
pub extern "C" fn init() {
    verify_admin_account();

    storage::new_dictionary(DATA_DICT).unwrap_or_default();
}

#[no_mangle]
pub extern "C" fn call() {
    let name: String = runtime::get_named_arg(NAME);

    let insert_entry_point: EntryPoint = EntryPoint::new(
        ENTRY_POINT_INSERT,
        vec![Parameter::new(DATA, CLType::String)],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let init_entry_point: EntryPoint = EntryPoint::new(
        ENTRY_POINT_INIT,
        vec![],
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract
    );

    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(insert_entry_point);
    entry_points.add_entry_point(init_entry_point);

    let mut named_keys = NamedKeys::new();
    let caller = runtime::get_caller();

    named_keys.insert(String::from(NAME), storage::new_uref(name).into());
    named_keys.insert(String::from(DATA_COUNT), storage::new_uref(0u64).into());
    named_keys.insert(String::from(OWNER), storage::new_uref(caller).into());

    let now: u64 = runtime::get_blocktime().into();
    let str1 = &now.to_string();

    let str2 = String::from("raffles_store_package_hash_");
    let str3 = String::from("raffles_store_access_uref_");
    let str4 = String::from("raffles_store_contract_hash_");
    let hash_name = str2 + &str1;
    let uref_name = str3 + &str1;
    let contract_hash_text = str4 + &str1;

    let (contract_hash, _contract_version) = storage::new_contract(
        entry_points,
        Some(named_keys),
        Some(hash_name.to_string()),
        Some(uref_name.to_string())
    );

    runtime::put_key(&contract_hash_text.to_string(), contract_hash.into());

    runtime::call_contract::<()>(contract_hash, ENTRY_POINT_INIT, runtime_args! {})
}

pub fn get_key<T: FromBytes + CLTyped>(name: &str) -> T {
    let key = runtime::get_key(name).unwrap_or_revert().try_into().unwrap_or_revert();
    storage::read(key).unwrap_or_revert().unwrap_or_revert()
}

fn verify_admin_account() {
    let admin: AccountHash = get_key(OWNER);
    let caller = runtime::get_caller();
    if admin != caller {
        runtime::revert(Error::AdminError);
    }
}

#[repr(u16)]
pub enum Error {
    AdminError = 1,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}
