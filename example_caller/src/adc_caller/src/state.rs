use candid::Principal;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub static IC_ADC_CANISTER: RefCell<Option<Principal>> = RefCell::default();
    pub static FEE: RefCell<u128> = RefCell::default();
    pub static REQUEST_MAP: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Getter for `IC_ADC_CANISTER` state variable
pub fn get_adc_address() -> Option<Principal> {
    let verifier_canister = IC_ADC_CANISTER.with(|vc| vc.borrow().clone());
    verifier_canister
}

/// Getter for `FEE` state variable
pub fn get_transaction_fee() -> u128 {
    let fee = FEE.with(|vc| vc.borrow().clone());
    fee
}

/// Setter for `IC_ADC_CANISTER` state variable
pub fn set_adc_address(new_adc_addressr: Option<Principal>) {
    IC_ADC_CANISTER
        .with(|old_adc_addressr| *old_adc_addressr.borrow_mut() = new_adc_addressr);
}


/// Setter for `FEE` state variable
pub fn set_transaction_fee(new_fee: u128) {
    FEE.with(|old_fee| *old_fee.borrow_mut() = new_fee);
}

/// Getter for the `REQUEST_MAP` state variable.
/// Returns a clone of the string value associated with the provided key, if it exists.
pub fn get_request_value(key: &str) -> Option<String> {
    REQUEST_MAP.with(|map| map.borrow().get(key).cloned())
}

/// Setter for the `REQUEST_MAP` state variable.
/// Inserts or updates the value for the provided key.
pub fn set_request_value(key: &str, value: String) {
    REQUEST_MAP.with(|map| {
        map.borrow_mut().insert(key.to_string(), value);
    });
}