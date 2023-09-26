#![no_std]
#![no_main]

// #[cfg(not(target_arch = "wasm32"))]
// compile_error!("target arch should be wasm32: compile with '--target wasm32-unknown-unknown'");

use casper_contract::{
   contract_api::{account, runtime, system},
   unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{runtime_args, ContractHash, RuntimeArgs, URef, Key, U512 , account::AccountHash};

const RAFFLE_CONTRACT_HASH: &str = "raffle_contract_hash";
const ENTRY_POINT_GET_PRICE: &str = "get_price";
const ENTRY_POINT_GET_PURSE: &str = "get_purse";
const ENTRY_POINT_BUY_TICKET: &str = "buy_ticket";

#[no_mangle]
pub extern "C" fn call() {
   let raffle_contract_hash: ContractHash = runtime::get_named_arg(RAFFLE_CONTRACT_HASH);
   let amount : U512 = runtime::get_named_arg("amount");

//  let amount: U512 = runtime::call_contract(raffle_contract_hash,ENTRY_POINT_GET_PRICE,runtime_args! {});
    let deposit_purse: URef = runtime::call_contract(raffle_contract_hash,ENTRY_POINT_GET_PURSE,runtime_args! {});

    let caller : AccountHash = runtime::get_caller().into();
   // Transfer from the caller's main purse to the new purse that was just created.
   // Note that transfer is done safely by the host logic.
   system::transfer_from_purse_to_purse(account::get_main_purse(), deposit_purse, amount, None)
       .unwrap_or_revert();

    runtime::call_contract::<()>(raffle_contract_hash,ENTRY_POINT_BUY_TICKET,runtime_args! {
        "partipiciant" => Key::Account(caller),
    });

}
