use candid::Principal;
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    pub static REQUEST_RESPONSE_BUFFER: RefCell<HashMap<String, bool>> = RefCell::default();
    pub static VERIFIER_CANISTER: RefCell<Option<Principal>> = RefCell::default();
}

/// Getter for the `REQUEST_RESPONSE_BUFFER` state variable
pub fn get_buffer() -> HashMap<String, bool> {
    REQUEST_RESPONSE_BUFFER.with(|rc| rc.borrow().clone())
}

/// Setter for the `REQUEST_RESPONSE_BUFFER` state variable
pub fn set_buffer(buffer: HashMap<String, bool>) {
    REQUEST_RESPONSE_BUFFER.with(|store| *store.borrow_mut() = buffer);
}

/// Getter for `VERIFIER_CANISTER` state variable
pub fn get_verifier_canister() -> Option<Principal> {
    let verifier_canister = VERIFIER_CANISTER.with(|vc| vc.borrow().clone());

    verifier_canister
}

/// Setter for `VERIFIER_CANISTER` state variable
pub fn set_verifier_canister(new_verifier_canister: Option<Principal>) {
    VERIFIER_CANISTER
        .with(|old_verifier_canister| *old_verifier_canister.borrow_mut() = new_verifier_canister);
}
