use ic_cdk::api::call;
use ic_cdk_macros::{update, export_candid};
use ic_principal::Principal;

#[update]
async fn display_info(name_canister_id: Principal, age_canister_id: Principal) -> String {
    // Call name_canister.get_name()
    let (name,): (String,) = call::call(name_canister_id, "get_name", ())
        .await
        .expect("Failed to call get_name");

    // Call age_canister.get_age()
    let (age,): (u32,) = call::call(age_canister_id, "get_age", ())
        .await
        .expect("Failed to call get_age");

    format!("Name: {}, Age: {}", name, age)
}

export_candid!();