use crate::{
    config::env_config::Config, services::trusted_registry::trusted_registry::Contract as C,
    utils::utils::Utils,
};
use anyhow::anyhow;
use sha3::{Digest, Keccak256};
use web3::{
    ethabi::{Bytes, Contract, Event, Hash, Log, RawLog},
    transports::Http,
    types::{BlockNumber, FilterBuilder, H160, H256, U64},
    Web3,
};

pub struct EventManager {
    abi: String,
    web3: Web3<Http>,
    address: Vec<H160>,
}

impl EventManager {
    pub fn new(abi: String, params: C) -> anyhow::Result<EventManager> {
        let rpc_url = Config::get_provider(params.chain_id.clone());
        let http = web3::transports::Http::new(&rpc_url)?;
        let web3 = web3::Web3::new(http);
        let mut address = Vec::new();
        address.push(params.contract_address);
        Ok(EventManager { abi, web3, address })
    }

    fn wrap_signature(&self, event: Event) -> anyhow::Result<Option<Vec<H256>>> {
        let signature = event.signature();
        let topic: Vec<H256> = vec![signature];
        Ok(Some(topic))
    }

    pub async fn sweep(
        &self,
        from: &str,
        to: &str,
        name_or_signature: &str,
    ) -> anyhow::Result<Vec<Log>> {
        let event = self.load_event(&self.abi, &name_or_signature)?;
        let wrapped_topic = self.wrap_signature(event.clone()).unwrap(); // todo: improve
        let from = Utils::integer_part(&from).unwrap();
        let to = Utils::integer_part(&to).unwrap();
        let filter = FilterBuilder::default()
            .address(self.address.clone())
            .topics(wrapped_topic, None, None, None)
            .from_block(BlockNumber::Number(U64::from(from)))
            .to_block(BlockNumber::Number(U64::from(to)))
            .build();
        let filter = self.web3.eth_filter().create_logs_filter(filter).await?;
        let logs = filter.logs().await.unwrap();
        let result = logs
            .into_iter()
            .map(|log| {
                let mapped_topics = log
                    .topics
                    .into_iter()
                    .map(|topic| Hash::from_slice(topic.as_bytes()))
                    .collect::<Vec<_>>();
                match event.parse_log(RawLog {
                    topics: mapped_topics,
                    data: Bytes::from(log.data.0),
                }) {
                    Ok(log) => log,
                    Err(e) => panic!("{}", e),
                }
            })
            .collect::<Vec<_>>();
        Ok(result)
    }

    fn load_event(&self, abi: &str, name_or_signature: &str) -> anyhow::Result<Event> {
        let contract: Contract = serde_json::from_str(abi).unwrap();
        let params_start = name_or_signature.find('(');
        match params_start {
            Some(params_start) => {
                let name = &name_or_signature[..params_start];
                let signature = Hash::from_slice(
                    Keccak256::digest(name_or_signature.replace(' ', "").as_bytes()).as_slice(),
                );
                contract
                    .events_by_name(name)?
                    .iter()
                    .find(|event| event.signature() == signature)
                    .cloned()
                    .ok_or_else(|| anyhow!("Invalid Signature `{}`", signature))
            }
            None => {
                let events = contract.events_by_name(name_or_signature)?;
                match events.len() {
                    0 => unreachable!(),
                    1 => Ok(events[0].clone()),
                    _ => Err(anyhow!(
                        "More than one function found for name `{}`, try providing full signature",
                        name_or_signature
                    )),
                }
            }
        }
    }
}
