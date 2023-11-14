use crate::{ alloc::string::ToString, utils::get_current_address };
use alloc::{ collections::BTreeMap, vec::Vec };
use casper_contract::contract_api::storage;
use casper_types::{ URef, Key };

pub enum RaffleEvent {
    BuyTicket {
        partipiciant: Key,
    },
    Draw {
        winner: u64,
    },
    Claim {
        winner_partipiciant: Key,
        collection: Key,
        token_id: u64,
    },
}

pub fn emit(event: &RaffleEvent) {
    let mut events = Vec::new();
    let mut param = BTreeMap::new();
    param.insert(
        "contract_package_hash",
        get_current_address().as_contract_package_hash().unwrap().to_string()
    );
    match event {
        RaffleEvent::BuyTicket { partipiciant } => {
            param.insert("event_type", "buy_ticket".to_string());
            param.insert("partipiciant", partipiciant.to_string());
        }
        RaffleEvent::Draw { winner } => {
            param.insert("event_type", "draw".to_string());
            param.insert("winner", winner.to_string());
        }
        RaffleEvent::Claim { winner_partipiciant, collection, token_id } => {
            param.insert("event_type", "claim".to_string());
            param.insert("winner_partipiciant", winner_partipiciant.to_string());
            param.insert("collection", collection.to_string());
            param.insert("token_id", token_id.to_string());
        }
    }
    events.push(param);
    for param in events {
        let _: URef = storage::new_uref(param);
    }
}
