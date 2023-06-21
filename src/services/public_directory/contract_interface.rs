use web3::{
    contract::{Contract, Options},
    ethabi::Log,
    transports::Http,
    types::U256,
};

use crate::{
    config::env_config::Config,
    services::{trusted_registry::trusted_registry::Contract as C, web3::event::EventManager},
};

use std::str;

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
    pub async fn get_last_block(&self) -> anyhow::Result<i64> {
        let result = self
            .contract_instance
            .query("prevBlock", (), None, Options::default(), None);
        let prev_block: U256 = result.await?;
        Ok(i64::from(prev_block.as_u32()))
    }

    pub async fn get_events_in_block_by_method(
        &self,
        name_or_signature: &str,
        block: &str,
    ) -> anyhow::Result<Vec<Log>> {
        let member_changed = self
            .event_manager
            .sweep(block, block, name_or_signature)
            .await?;
        Ok(member_changed)
    }
}
