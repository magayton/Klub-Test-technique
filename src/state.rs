use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

//Information generale du contrat
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateInfo {
    //L'instantiateur du contrat sera l'admin
    pub admin_addr: Addr, 
    pub cfo_addr: Addr,
    //Token accepte pour le paiment du contrat
    pub token_denom: String, 
    pub min_withdrawal: Uint128, 
}

//Un client represente par son nombre de token lock et son yield genere
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Client {
   pub nb_token_staked: Uint128, 
   pub yield_generated: Uint128,
}

//Etat general de notre contrat : nombre de token lock avec yield, nombre de token lock sans yield
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Pool {
    pub pool_total_amount: Uint128,
    pub pool_total_amount_staked: Uint128,
    //Le nombre total de token en cours de claim
    pub total_claim: Uint128,
}

//Vecteur contenant toutes les adresses des clients ayant des tokens lock
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct ClientsList {
    pub clients_list: Vec<Addr>,
}

pub const STATE: Item<StateInfo> = Item::new("state");
pub const POOL: Item<Pool> = Item::new("pool");
pub const CLIENTS: Map<Addr, Client> = Map::new("clients");
pub const CLIENTS_LIST: Item<ClientsList> = Item::new("clients_list");
