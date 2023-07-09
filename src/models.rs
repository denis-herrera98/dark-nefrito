use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DarkSoulsUsers {
    deaths: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DarkSoulsData {
    pub users: HashMap<String, DarkSoulsUsers>,
}
