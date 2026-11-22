mod ui;

use std::{sync::Arc, thread};

use hub_core::{Io, Ports, SystemClock, UiState};
use slint::ComponentHandle;
use tokio::sync::{oneshot, watch};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    let weak = app.as_weak();
    let core = thread::spawn(move || -> anyhow::Result<()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move {
            ui::spawn_bridge(weak, ui_rx);
            let ports = Ports {
                clock: Arc::new(SystemClock),
            };
            tokio::select! {
                result = hub_core::run(ports, Io { ui_tx }) => result,
                _ = stop_rx => Ok(()),
            }
        })
    });

    app.run()?;
    let _ = stop_tx.send(());
    core.join()
        .map_err(|_| anyhow::anyhow!("hub-core thread panicked"))?
}
