use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::U256,
};

use crate::{
    config::env_config::Config, services::trusted_registry::trusted_registry::Contract as C,
};

pub struct ContractInterface {
    contract_instance: Contract<Http>,
}

impl ContractInterface {
    pub async fn new(params: C) -> anyhow::Result<ContractInterface> {
        let rpc_url = Config::get_provider(params.chain_id);
        let http = web3::transports::Http::new(&rpc_url)?;
        let web3 = web3::Web3::new(http);
        let abi = include_bytes!("./abi.json");
        let contract_instance = Contract::from_json(web3.eth(), params.contract_address, abi)?;
        Ok(ContractInterface { contract_instance })
    }
    pub async fn get_last_block(&self) -> anyhow::Result<i64> {
        let result = self
            .contract_instance
            .query("prevBlock", (), None, Options::default(), None);
        let prev_block: U256 = result.await?;
        Ok(i64::from(prev_block.as_u32()))
    }
}
