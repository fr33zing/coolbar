use anyhow::{anyhow, Result};
use rand::{rngs::SmallRng, SeedableRng};
use relm4::{
    gtk::{
        gio::{Cancellable, DBusConnection, DBusMessage, DBusSendMessageFlags, DBusSignalFlags},
        glib::Variant,
    },
    Reducer, Reducible,
};
use tokio::{sync::OnceCell, task};
use tracing::{debug, error, trace};

use crate::{config, dbus::wait_for_dbus};

const OPENRAZER_BUS_NAME: Option<&str> = Some("org.razer");

pub static REDUCER: Reducer<OpenRazerReducer> = Reducer::new();
static DBUS: OnceCell<&DBusConnection> = OnceCell::const_new();

#[derive(Default, Debug, Clone)]
pub struct OpenRazerReducer {
    pub mouse_error: Option<String>,
    pub mouse_detected: bool,
    pub mouse_charging: bool,
    pub mouse_battery_level: f64,
}

#[derive(Debug)]
pub enum OpenRazerInput {
    MouseError(String),
    MouseDetected(bool),
    MouseBattery { charging: bool, battery_level: f64 },
    DeviceAdded,
    DeviceRemoved,
}

impl Reducible for OpenRazerReducer {
    type Input = OpenRazerInput;

    fn init() -> Self {
        task::spawn(async {
            if let Err(err) = connect().await {
                error!("dbus connection failed: {err}");
            }
        });

        Self::default()
    }

    fn reduce(&mut self, input: Self::Input) -> bool {
        match input {
            OpenRazerInput::MouseError(err) => {
                error!({ err }, "failed to update mouse battery level");
                self.mouse_error = Some(err);
            }
            OpenRazerInput::MouseDetected(detected) => {
                self.mouse_detected = detected;
                if !detected {
                    self.mouse_charging = false;
                }
            }
            OpenRazerInput::MouseBattery {
                charging,
                battery_level: level,
            } => {
                self.mouse_detected = true;
                self.mouse_charging = charging;
                self.mouse_battery_level = level;
            }
            OpenRazerInput::DeviceAdded | OpenRazerInput::DeviceRemoved => {
                debug!(
                    { reason = format!("{input:?}") },
                    "updating battery level early"
                );
                let dbus = DBUS.get().expect("no dbus connection available");
                if let Err(err) = update_mouse(dbus) {
                    REDUCER.emit(OpenRazerInput::MouseError(err.to_string()));
                }
            }
        }
        true
    }
}

fn call_dbus_method(dbus: &DBusConnection, message: &DBusMessage) -> Result<Variant> {
    let reply = dbus.send_message_with_reply_sync(
        &message,
        DBusSendMessageFlags::NONE,
        5000,
        Cancellable::NONE,
    )?;
    let body = reply
        .0
        .body()
        .ok_or_else(|| anyhow!("failed to get reply body"))?;
    Ok(body)
}

async fn connect() -> Result<()> {
    trace!("waiting for dbus connection...");
    let dbus = wait_for_dbus().await?;
    DBUS.set(dbus)?;
    trace!("got dbus connection");

    trace!("subscribing to openrazer dbus events");
    dbus.signal_subscribe(
        OPENRAZER_BUS_NAME,
        Some("razer.devices"),
        Some("device_added"),
        Some("/org/razer"),
        None,
        DBusSignalFlags::NONE,
        |_, _, _, _, _, _| REDUCER.emit(OpenRazerInput::DeviceAdded),
    );
    dbus.signal_subscribe(
        OPENRAZER_BUS_NAME,
        Some("razer.devices"),
        Some("device_removed"),
        Some("/org/razer"),
        None,
        DBusSignalFlags::NONE,
        |_, _, _, _, _, _| REDUCER.emit(OpenRazerInput::DeviceRemoved),
    );

    trace!("beginning device battery polling loop");
    let mut rng = SmallRng::from_entropy();
    loop {
        if let Err(err) = update_mouse(dbus) {
            REDUCER.emit(OpenRazerInput::MouseError(err.to_string()));
            return Err(err);
        }
        let delay = config::get()
            .providers
            .openrazer
            .polling_rate
            .to_duration(&mut rng);
        tokio::time::sleep(delay).await;
    }
}

fn update_mouse(dbus: &DBusConnection) -> Result<()> {
    let devices = {
        let message = DBusMessage::new_method_call(
            OPENRAZER_BUS_NAME,
            "/org/razer",
            Some("razer.devices"),
            "getDevices",
        );

        call_dbus_method(dbus, &message)?
            .try_child_get::<Vec<String>>(0)?
            .ok_or_else(|| anyhow!("failed to get razer devices"))?
    };

    if devices.len() == 0 {
        trace!("no razer devices found");
        REDUCER.emit(OpenRazerInput::MouseDetected(false));
        return Ok(());
    }

    let mouse = devices.iter().find(|device| {
        let message = DBusMessage::new_method_call(
            OPENRAZER_BUS_NAME,
            &format!("/org/razer/device/{device}"),
            Some("razer.device.misc"),
            "getDeviceType",
        );
        let Ok(reply) = call_dbus_method(dbus, &message) else {
            return false;
        };
        let Ok(device_type) = reply.try_child_get::<String>(0) else {
            return false;
        };
        let Some(device_type) = device_type else {
            return false;
        };

        device_type == "mouse"
    });
    let Some(mouse) = mouse else {
        trace!("no mouse found");
        REDUCER.emit(OpenRazerInput::MouseDetected(false));
        return Ok(());
    };

    let charging = {
        let message = DBusMessage::new_method_call(
            OPENRAZER_BUS_NAME,
            &format!("/org/razer/device/{mouse}"),
            Some("razer.device.power"),
            "isCharging",
        );

        call_dbus_method(dbus, &message)?
            .try_child_get::<bool>(0)?
            .ok_or_else(|| anyhow!("failed to get mouse charging status"))?
    };

    let battery_level = {
        let message = DBusMessage::new_method_call(
            OPENRAZER_BUS_NAME,
            &format!("/org/razer/device/{mouse}"),
            Some("razer.device.power"),
            "getBattery",
        );

        call_dbus_method(dbus, &message)?
            .try_child_get::<f64>(0)?
            .ok_or_else(|| anyhow!("failed to get mouse battery level"))?
    };

    REDUCER.emit(OpenRazerInput::MouseBattery {
        charging,
        battery_level,
    });

    Ok(())
}
