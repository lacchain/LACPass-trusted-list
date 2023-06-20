use std::{thread, time::Duration};

use log::info;
use yansi::Paint;

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
                                let message = format!(
                                    "{} {} {} {}",
                                    Paint::masked("🌀"),
                                    Paint::green(
                                        "Sucessful public key update, next update will take place in"
                                    )
                                    .bold(),
                                    r.period_seconds,
                                    "seconds..."
                                );
                                info!("{}", message);
                                tokio::time::sleep(Duration::from_secs(r.period_seconds)).await;
                            }
                            Err(_e) => {
                                let message = format!("{} {} {}", Paint::masked("❌") ,Paint::red("Failed to sweep, ... retrying in "), r.retry_period);
                                error!(
                                    "{}", message
                                );
                                tokio::time::sleep(Duration::from_secs(r.retry_period)).await;
                            }
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
    }
}
