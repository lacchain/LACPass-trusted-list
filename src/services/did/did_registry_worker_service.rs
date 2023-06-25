use crate::{
    entities::models::DidModel,
    services::{
        did::data_interface::DidDataInterfaceService,
        public_key::data_interface::PublicKeyService,
        trusted_registry::trusted_registry::Contract,
        web3::utils::{
            get_address_from_log, get_bool_from_log, get_bytes_from_log, get_u64_from_log,
        },
    },
};
use crypto::{digest::Digest, sha3::Sha3};
use log::{debug, info};
use sea_orm::DatabaseConnection;
use web3::ethabi::Log;

use super::index::{DidLac1, DidService};

pub struct DidRegistryWorkerService {
    did_service: DidService,
    public_key_service: PublicKeyService,
    did: DidModel,
    did_params: DidLac1,
}

impl DidRegistryWorkerService {
    pub async fn new(did: DidModel) -> anyhow::Result<Self> {
        match DidService::decode_did(&did.did) {
            Ok(did_params) => {
                let params = Contract {
                    chain_id: did_params.chain_id.to_string(),
                    contract_address: did_params.did_registry_address,
                };
                match DidService::new(params).await {
                    Ok(did_service) => Ok(Self {
                        did_service,
                        public_key_service: PublicKeyService::new(),
                        did,
                        did_params,
                    }),
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    pub async fn exec_or_resume_scheduled_sweep(
        &mut self,
        db: &DatabaseConnection,
    ) -> anyhow::Result<()> {
        info!("excuting scheduled worker thread");
        if self.did.upper_block == self.did.last_block_saved {
            info!("All up to date in this scheduled task");
            return Ok(());
        }

        if self.did.upper_block >= self.did.last_processed_block
            && self.did.last_processed_block > self.did.last_block_saved
        {
            info!("Found not finished job, resuming");
            match self
                .did_service
                .contract_interface_service
                .find_previous_block(&(self.did.last_processed_block as u64))
                .await
            {
                Ok(u) => {
                    if let Some(prev_to_last_processed_block) = u {
                        return self
                            .process_events_in_block_range(
                                db,
                                prev_to_last_processed_block,
                                self.did.last_block_saved as u64,
                            )
                            .await;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        info!("Starting scheduled job");
        return self
            .process_events_in_block_range(
                db,
                self.did.upper_block as u64,
                self.did.last_block_saved as u64,
            )
            .await;
    }

    pub async fn process_events_in_block_range(
        &mut self,
        db: &DatabaseConnection,
        contract_block: u64,
        target_block: u64,
    ) -> anyhow::Result<()> {
        let mut block_to_process = contract_block;
        while block_to_process > target_block {
            info!(
                "Starting sweep; from block {}, to target block  {}",
                &block_to_process, &target_block
            );
            match self.process_events_in_block(db, &block_to_process).await {
                Ok(prev_block) => {
                    match DidDataInterfaceService::update(
                        db,
                        None,
                        Some(block_to_process),
                        None,
                        &self.did.did,
                    )
                    .await
                    {
                        Ok(v) => {
                            self.did = v;
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                    block_to_process = prev_block;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        if block_to_process == target_block {
            match DidDataInterfaceService::update(
                db,
                None,
                Some(0),
                Some(contract_block),
                &self.did.did,
            )
            .await
            {
                Ok(v) => {
                    self.did = v;
                    info!("Reached target block {}", block_to_process);
                    Ok(())
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        } else {
            panic!("Unexpected values, last block saved on database:{}, is greater than passed contract block: {}", &target_block, &block_to_process);
        }
    }

    //// Process event in the block whose number is passed as an argument.
    /// Returns previous block
    pub async fn process_events_in_block(
        &self,
        db: &DatabaseConnection,
        block: &u64,
    ) -> anyhow::Result<u64> {
        let prev_block: u64;

        // TODO: make static
        match self
            .did_service
            .contract_interface_service
            .get_events_in_block_by_method("DIDAttributeChanged", &block)
            .await
        {
            Ok(did_attribute_changed_logs) => {
                match self
                    .process_did_attribute_changed_event(db, did_attribute_changed_logs, block)
                    .await
                {
                    Err(e) => return Err(e.into()),
                    Ok(v) => {
                        prev_block = v;
                    }
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
        Ok(prev_block)
    }

    pub async fn process_did_attribute_changed_event(
        &self,
        db: &DatabaseConnection,
        did_attribute_changed_logs: Vec<Log>,
        block: &u64,
    ) -> anyhow::Result<u64> {
        let mut prev_block: u64 = 0;
        for did_attribute_changed_log in did_attribute_changed_logs {
            let identity = get_address_from_log(&did_attribute_changed_log, "identity");
            if identity != self.did_params.address {
                info!(
                    "Skipping log. Identities doesn't match, found identity {}; required {}",
                    identity, self.did_params.address
                );
                continue;
            }
            let name = get_bytes_from_log(&did_attribute_changed_log, "name");
            match String::from_utf8(name) {
                Ok(v) => {
                    info!("found new candidate public key for did {}", self.did.did);
                    let error_message = format!(
                        "Found public key for did {}, but params are unsupported",
                        self.did.did
                    );
                    if let [asse, _, algorithm, encoding_method] =
                        v.split('/').collect::<Vec<_>>().as_slice()
                    {
                        let is_candidate =
                            asse == &"asse" && algorithm == &"jwk" && encoding_method == &"cbor";
                        if !is_candidate {
                            info!("{}", error_message);
                            continue;
                        }
                    } else {
                        info!("{}", error_message);
                        continue;
                    }
                }
                Err(e) => {
                    info!(
                        "Unable to process public key related did {:?}. Error is: {:?}... skipping this registry",
                        self.did.did,
                        e
                    );
                    continue;
                }
            }
            let pem_key = get_bytes_from_log(&did_attribute_changed_log, "value");
            let valid_to = get_u64_from_log(&did_attribute_changed_log, "validTo");
            // let change_time = get_u64_from_log(&did_attribute_changed_log, "changeTime"); // Not needed for this logic
            prev_block = get_u64_from_log(&did_attribute_changed_log, "previousChange");
            let is_compromised = get_bool_from_log(&did_attribute_changed_log, "compromised"); // TODO: analyze how to serve this

            let mut h = Sha3::keccak256();
            h.input(&pem_key);
            let content_hash = h.result_str();

            match self
                .public_key_service
                .find_public_key_by_content_hash(db, &content_hash, &self.did.id)
                .await
            {
                Ok(wrapped) => match wrapped {
                    Some(found_public_key) => {
                        if (found_public_key.block_number as u64) < *block {
                            match self
                                .public_key_service
                                .update_public_key(
                                    db,
                                    &found_public_key.id,
                                    Some(*block),
                                    Some(valid_to),
                                    Some(is_compromised),
                                )
                                .await
                            {
                                Ok(_) => {
                                    info!(
                                        "Updated public key with id: {:} for did: {}",
                                        found_public_key.id, self.did.did
                                    );
                                }
                                Err(e) => return Err(e.into()),
                            }
                        }
                    }
                    None => {
                        match self
                            .public_key_service
                            .insert_public_key(
                                db,
                                &self.did.id,
                                &block,
                                pem_key,
                                &content_hash,
                                &valid_to,
                                is_compromised,
                            )
                            .await
                        {
                            Ok(_) => {
                                info!("Inserted new public key for did: {}", self.did.did);
                            }
                            Err(e) => return Err(e.into()),
                        }
                    }
                },
                Err(e) => return Err(e.into()),
            }
        }
        Ok(prev_block)
    }

    pub async fn sweep(&mut self, db: &DatabaseConnection) -> anyhow::Result<()> {
        info!("Starting DidRegistryWorkerService sweep");
        debug!("Scanned did {} {}", self.did.did, self.did.id);
        match self.exec_or_resume_scheduled_sweep(db).await {
            Ok(_) => {
                let contract_last_block: u64;
                match self
                    .did_service
                    .contract_interface_service
                    .get_last_block(self.did_params.address)
                    .await
                {
                    Ok(result) => {
                        if result == 0 {
                            info!(
                                "No events found for did: {:?}... skipping sweep",
                                self.did.did
                            );
                            return Ok(());
                        }

                        contract_last_block = result;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
                // set upper block to contract last saved block
                // set last processed block to zero value
                match DidDataInterfaceService::get_did_from_database(db, &self.did.did).await {
                    Ok(v) => {
                        match v {
                            Some(m) => {
                                // verify whether insert is needed
                                if contract_last_block > 0
                                    && contract_last_block == m.upper_block as u64
                                    && contract_last_block == m.last_block_saved as u64
                                {
                                    info!("There are no changes in the contract. Last block saved is {}", contract_last_block);
                                    return Ok(());
                                }
                                match DidDataInterfaceService::update(
                                    db,
                                    Some(contract_last_block),
                                    Some(0),
                                    None,
                                    &self.did.did,
                                )
                                .await
                                {
                                    Ok(v) => {
                                        self.did = v.clone();
                                        self.exec_or_resume_scheduled_sweep(db).await
                                    }
                                    Err(err) => {
                                        return Err(err.into());
                                    }
                                }
                            }
                            None => {
                                info!("Initializing metadata for contract in database");
                                match DidDataInterfaceService::insert_did_to_database(
                                    db,
                                    &self.did.did,
                                    Some(contract_last_block),
                                    Some(0),
                                    Some(0),
                                )
                                .await
                                {
                                    Ok(v) => {
                                        self.did = v;
                                        self.exec_or_resume_scheduled_sweep(db).await
                                    }
                                    Err(e) => {
                                        return Err(e.into());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
