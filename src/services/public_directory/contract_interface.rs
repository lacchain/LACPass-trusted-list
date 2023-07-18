use web3::{
    contract::{Contract, Options},
    ethabi::Log,
    transports::Http,
    types::U256,
};

use crate::{
    config::env_config::Config,
    services::{
        trusted_registry::trusted_registry::Contract as C,
        web3::{event::EventManager, utils::get_u64_from_log},
    },
};

use std::str;

#[derive(Debug, Clone)]
pub struct ContractInterface {
    contract_instance: Contract<Http>,
    event_manager: EventManager,
}

impl ContractInterface {
    pub async fn new(params: C) -> anyhow::Result<ContractInterface> {
        let rpc_url = Config::get_provider(params.chain_id.clone());
        let http = web3::transports::Http::new(&rpc_url)?;
        let web3 = web3::Web3::new(http);
        let abi = include_bytes!("./abi.json");
        let contract_instance =
            Contract::from_json(web3.eth(), params.contract_address.clone(), abi)?;
        let str_abi = match str::from_utf8(abi) {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        };
        let event_manager = EventManager::new(str_abi.to_owned(), params)?;
        Ok(ContractInterface {
            contract_instance,
            event_manager,
        })
    }
    pub async fn get_last_block(&self) -> anyhow::Result<u64> {
        let result = self
            .contract_instance
            .query("prevBlock", (), None, Options::default(), None);
        let prev_block: U256 = result.await?;
        Ok(prev_block.as_u64())
    }

    /// Returns block previous prior to the last block saved param on the smart contract
    pub async fn find_previous_block(&self, block: &u64) -> anyhow::Result<Option<u64>> {
        match self
            .find_previous_block_by_event_name("MemberChanged", block)
            .await
        {
            Ok(v) => {
                if let Some(s) = v {
                    return Ok(Some(s));
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        match self
            .find_previous_block_by_event_name("DidAssociated", block)
            .await
        {
            Ok(v) => {
                if let Some(s) = v {
                    return Ok(Some(s));
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        match self
            .find_previous_block_by_event_name("DidDisassociated", block)
            .await
        {
            Ok(v) => {
                if let Some(s) = v {
                    return Ok(Some(s));
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        Ok(None)
    }

    pub async fn find_previous_block_by_event_name(
        &self,
        name_or_signature: &str,
        block: &u64,
    ) -> anyhow::Result<Option<u64>> {
        match self
            .get_events_in_block_by_method(name_or_signature, block)
            .await
        {
            Ok(logs) => {
                if logs.len() == 0 {
                    return Ok(None);
                }
                Ok(Some(get_u64_from_log(&logs[0], "prevBlock")))
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub async fn get_events_in_block_by_method(
        &self,
        name_or_signature: &str,
        block: &u64,
    ) -> anyhow::Result<Vec<Log>> {
        let member_changed = self
            .event_manager
            .sweep(block, block, name_or_signature)
            .await?;
        Ok(member_changed)
    }
}
