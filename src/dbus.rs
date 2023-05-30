use anyhow::{bail, Result};
use std::time::{Duration, Instant};

use relm4::gtk::gio::DBusConnection;
use tokio::{sync::OnceCell, time::sleep};

pub static DBUS_CONNECTION: OnceCell<DBusConnection> = OnceCell::const_new();

pub async fn wait_for_dbus() -> Result<&'static DBusConnection> {
    let now = Instant::now();
    loop {
        if let Some(dbus) = DBUS_CONNECTION.get() {
            return Ok(&dbus);
        }
        if now.elapsed().as_secs() > 5 {
            bail!("timed out while waiting for dbus connection");
        }
        sleep(Duration::from_millis(25)).await;
    }
}
