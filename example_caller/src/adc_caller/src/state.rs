use candid::Principal;
use std::cell::RefCell;

thread_local! {
    pub static IC_ADC_CANISTER: RefCell<Option<Principal>> = RefCell::default();
    pub static FEE: RefCell<u128> = RefCell::default();
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
