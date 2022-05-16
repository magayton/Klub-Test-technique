#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::to_binary;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};

use cw2::set_contract_version;
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info,
};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};


use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{StateInfo, Client, Pool, ClientsList, STATE, POOL, CLIENTS, CLIENTS_LIST};


const CONTRACT_NAME: &str = "crates.io:Klub-Deposit";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    //Creation du token CW20
    let data = TokenInfo {
        name: _msg.name,
        symbol: _msg.symbol,
        decimals: _msg.decimals,
        //Supply a 0 a la creation
        total_supply: Uint128::zero(),
        //Le contrat pourra mint les tokens a l'infini
        mint: Some(MinterData {
            minter: _env.contract.address,
            cap: None,
        }), 
    };
    TOKEN_INFO.save(_deps.storage, &data)?;

    //Si une adresse de cfo etait dans le message d'instation, il devient le cfo
    //Sinon l'admin du contrat est aussi le cfo
    let cfo = _msg.cfo.unwrap_or(_info.sender.to_string());
    let cfo_valid = _deps.api.addr_validate(&cfo)?;

    //Le token que le contrat accepte de recevoir 
    //J'ai laisse le "upebble" ici pour pouvoir tester
    //En production cela devrait etre le "ujuno"
    let payment_token = String::from("upebble");

    let state = StateInfo {
        admin_addr: _info.sender,
        cfo_addr: cfo_valid,
        token_denom: payment_token,
        min_withdrawal: _msg.min_withdrawal,
    };
    STATE.save(_deps.storage, &state)?;

    //On initialise par defaut (donc avec des 0)
    let pool = Pool::default();
    POOL.save(_deps.storage, &pool)?;

    let clients_list = ClientsList::default();
    CLIENTS_LIST.save(_deps.storage, &clients_list)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::Deposit{} => execute_deposit(_deps, _env, _info),

        //Fonctions pour le standard CW20
        ExecuteMsg::Transfer { recipient, amount } => Ok(execute_transfer(_deps, _env, _info, recipient, amount)?),
        ExecuteMsg::Burn { amount } => Ok(execute_burn(_deps, _env, _info, amount)?),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(_deps, _env, _info, contract, amount, msg)?),
    }
}

pub fn execute_deposit(deps: DepsMut, env: Env, info:MessageInfo) -> Result<Response, ContractError> {

    let state = STATE.load(deps.storage)?;

    //On verifie que la denom du token envoyer correspond a celle attendue
    let payment = info
        .funds
        .iter()
        .find(|x| x.denom == state.token_denom)
        .ok_or_else(|| ContractError::WrongPaymentTokenError{})?;
    
    //La quantite de token KJuno (CW20) a mint correspond a la quantite recu lors du paiement
    let quantity_to_mint = payment.amount;

    //Mise a jour des quantites de token dans la pool
    let mut pool = POOL.load(deps.storage)?;
    pool.pool_total_amount += quantity_to_mint;
    pool.pool_total_amount_staked += quantity_to_mint;
    POOL.save(deps.storage, &pool)?;

    //Creation d'un client
    if CLIENTS.has(deps.storage, info.sender.clone()) {

        //Si le client existe deja (a deja deposit avant) on ajoute le paiement a ses tokens lock
        let mut client_found = CLIENTS.load(deps.storage, info.sender.clone())?;
        client_found.nb_token_staked += quantity_to_mint;
        CLIENTS.save(deps.storage, info.sender.clone(), &client_found)?;

    } else {
        //Client qui n'a jamais deposit auparavant 
        //On ajoute son adresse a la liste des clients
        let mut clients_vec = CLIENTS_LIST.load(deps.storage)?;
        clients_vec.clients_list.push(info.sender.clone());
        CLIENTS_LIST.save(deps.storage, &clients_vec)?;

        //On cree un nouveau client 
        let client_not_found = Client {
            nb_token_staked : quantity_to_mint,
            yield_generated : Uint128::zero(),
        };

        CLIENTS.save(deps.storage, info.sender.clone(), &client_not_found)?;

    }

    let submessage_info = MessageInfo {
        sender: env.contract.address.clone(),
        funds: vec![],
    };
    
    //On mint directement le token CW20 (Kjuno) a l'adresse du client
    execute_mint(deps, env, submessage_info, info.sender.to_string(), quantity_to_mint)?;

    Ok(Response::new()
        .add_attribute("action", "Deposit")
        .add_attribute("quantity_minted", quantity_to_mint.to_string())
        .add_attribute("address_to_mint", info.sender.to_string())
    )

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    //Queries pour respecter le standard CW20
    match _msg {
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(_deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(_deps, address)?),
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, QueryMsg,
    };
    use crate::error::ContractError;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, Uint128, Coin, from_binary};
    use cw20_base::contract::{
        query_token_info,
    };

    use cw20::BalanceResponse;

    pub const ADDR1: &str = "wasm1p98s59lc86eycdnk09c0jhdv2p9k6m0hrcf4zs";

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);

        let msg = InstantiateMsg{
            name : String::from("KJuno"), 
            symbol : String::from("Klubj"), 
            decimals : 8, 
            cfo : None, 
            min_withdrawal: Uint128::from(5u128) 
        };

        let res = instantiate(deps.as_mut(), env, info, msg.clone()).unwrap();
        let token = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(res.attributes, vec![attr("action", "instantiate"),]);
        assert_eq!(&token.name, &msg.name);
        assert_eq!(&token.symbol, &msg.symbol);
        assert_eq!(token.decimals, msg.decimals);
        assert_eq!(token.total_supply, Uint128::zero());
        
    }
    #[test]
    fn test_execute_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin {
            denom : String::from("utokenfail"),
            amount : Uint128::from(100u128),
        }]);

        
        let msg = InstantiateMsg{
            name : String::from("KJuno"), 
            symbol : String::from("Klubj"), 
            decimals : 8, 
            cfo : None, 
            min_withdrawal: Uint128::from(5u128) 
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        let msg_execute = ExecuteMsg::Deposit {};

        let _err = execute(deps.as_mut(), env, info, msg_execute).unwrap_err();
        assert_eq!(_err, ContractError::WrongPaymentTokenError{});
    }

    #[test]
    fn test_execute_ok() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin {
            denom : String::from("upebble"),
            amount : Uint128::from(100u128),
        }]);

        
        let msg = InstantiateMsg{
            name : String::from("KJuno"), 
            symbol : String::from("Klubj"), 
            decimals : 8, 
            cfo : None, 
            min_withdrawal: Uint128::from(5u128) 
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        let msg_execute = ExecuteMsg::Deposit {};

        let res_execute = execute(deps.as_mut(), env.clone(), info, msg_execute).unwrap();
        assert_eq!(res_execute.attributes, vec![attr("action", "Deposit"), attr("quantity_minted", "100"), attr("address_to_mint", ADDR1),]);

        let msg_query = QueryMsg::Balance{address : String::from(ADDR1)};
        let bin = query(deps.as_ref(), env, msg_query).unwrap();
        let res : BalanceResponse = from_binary(&bin).unwrap();
        assert_eq!(res.balance, Uint128::from(100u128));
    }
}
