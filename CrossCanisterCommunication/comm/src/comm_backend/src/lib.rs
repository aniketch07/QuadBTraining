use ic_cdk_macros::{update, query, export_candid};
use std::cell::RefCell;

thread_local! {
    static NAME: RefCell<String> = RefCell::new(String::new());
}

#[update]
fn set_name(name: String) {
    NAME.with(|n| *n.borrow_mut() = name);
}

#[query]
fn get_name() -> String {
    NAME.with(|n| n.borrow().clone())
}

export_candid!();