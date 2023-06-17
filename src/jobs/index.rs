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
                        match r.sweep().await {
                            Ok(_) => {
                                info!(
                                    "Sucessful update, next update will take place in {:?} seconds ...",
                                    r.period_seconds
                                );
                                thread::sleep(Duration::from_secs(r.period_seconds));
                            }
                            Err(_e) => {
                                error!(
                                    "Failed sweep: {:?} ... retrying in {:?} seconds ...",
                                    r, r.retry_period
                                );
                                thread::sleep(Duration::from_secs(r.retry_period));
                            }
                        }
                    }
                })
            })
            //.map(|th| async { th.await.unwrap() })
            .collect::<Vec<_>>();
    }
}
