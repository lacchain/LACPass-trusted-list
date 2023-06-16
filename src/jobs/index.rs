use std::{thread, time::Duration};

use log::info;

use crate::jobs::trusted_registries::TrustedRegistries;

#[derive(Debug)]
pub struct JobManager {}
impl JobManager {
    pub fn sweep_trusted_registries() {
        info!("Starting Trusted Registres Worker");
        let _p = TrustedRegistries::new()
            .registries
            .into_iter()
            .map(|r| {
                tokio::spawn(async move {
                    thread::sleep(Duration::from_secs(r.start_up));
                    loop {
                        r.sweep();
                        thread::sleep(Duration::from_secs(r.period_seconds))
                    }
                })
            })
            .collect::<Vec<_>>();
    }
}
