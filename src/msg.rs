use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, Uint128};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    //Nom du token CW20 a creer
    pub name: String,
    //Ticker du token
    pub symbol: String,
    pub decimals: u8,
    //On peut optionnellement passer l'adresse du cfo
    pub cfo: Option<String>,
    pub min_withdrawal: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit{},

    //Fonctions pour respecter le standard CW20
    Transfer { recipient: String, amount: Uint128 },

    Burn { amount: Uint128 },

    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    //Queries pour respecter le standard CW20
    Balance { address: String },
    TokenInfo {},
}
