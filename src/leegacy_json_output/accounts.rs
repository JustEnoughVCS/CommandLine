use just_enough_vcs::vcs::data::member::MemberId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountListJsonResult {
    pub result: HashMap<MemberId, AccountItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountItem {
    pub has_private_key: bool,
}
