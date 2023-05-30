use std::time::Duration;

use anyhow::{anyhow, Result};
use relm4::{
    gtk::{
        gio::{Cancellable, DBusConnection, DBusMessage, DBusSendMessageFlags, DBusSignalFlags},
        glib::Variant,
    },
    Reducer, Reducible,
};
use tokio::{sync::OnceCell, task};

use crate::globals::wait_for_dbus;

const OPENRAZER_BUS_NAME: Option<&str> = Some("org.razer");
const BATTERY_POLL_RATE: Duration = Duration::from_secs(666);

pub static REDUCER: Reducer<OpenRazerReducer> = Reducer::new();

static DBUS: OnceCell<&DBusConnection> = OnceCell::const_new();

#[derive(Debug, Clone)]
pub struct OpenRazerReducer {
    pub mouse_detected: bool,
    pub mouse_battery: f64,
}

#[derive(Debug)]
pub enum OpenRazerInput {
    MouseDetected(bool),
    MouseBattery(f64),
    DeviceAdded,
    DeviceRemoved,
}

impl Reducible for OpenRazerReducer {
    type Input = OpenRazerInput;

    fn init() -> Self {
        task::spawn(async {
            if let Err(err) = connect().await {
                tracing::error!("dbus connection failed: {err}");
            }
        });

        Self {
            mouse_detected: false,
            mouse_battery: 0.0,
        }
    }

    fn reduce(&mut self, input: Self::Input) -> bool {
        match input {
            OpenRazerInput::MouseDetected(detected) => {
                self.mouse_detected = detected;
            }
            OpenRazerInput::MouseBattery(battery) => {
                self.mouse_detected = true;
                self.mouse_battery = battery;
            }
            OpenRazerInput::DeviceAdded | OpenRazerInput::DeviceRemoved => {
                tracing::debug!(
                    { reason = format!("{:?}", input) },
                    "updating battery level early"
                );
                update_mouse_battery_level(DBUS.get().expect("no dbus connection available"))
                    .unwrap(); // TODO fix me
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
    tracing::trace!("waiting for dbus connection...");
    let dbus = wait_for_dbus().await?;
    DBUS.set(dbus)?;
    tracing::trace!("got dbus connection");

    tracing::trace!("subscribing to openrazer dbus events");
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

    tracing::trace!("beginning device battery polling loop");
    loop {
        if let Err(err) = update_mouse_battery_level(dbus) {
            tracing::error!("failed to update mouse battery level: {err}");
            return Err(err);
        }
        tokio::time::sleep(BATTERY_POLL_RATE).await;
    }
}

fn update_mouse_battery_level(dbus: &DBusConnection) -> Result<()> {
    tracing::trace!("getting razer devices");
    let devices = {
        let message = DBusMessage::new_method_call(
            OPENRAZER_BUS_NAME,
            "/org/razer",
            Some("razer.devices"),
            "getDevices",
        );
        let reply = call_dbus_method(dbus, &message)?;

        reply
            .try_child_get::<Vec<String>>(0)?
            .ok_or_else(|| anyhow!("failed to get razer devices"))?
    };
    tracing::trace!("got {} razer device(s)", devices.len());

    tracing::trace!("finding first mouse");
    let mouse = devices.iter().find(|device| {
        let message = DBusMessage::new_method_call(
            OPENRAZER_BUS_NAME,
            &format!("/org/razer/device/{device}"),
            Some("razer.device.misc"),
            "getDeviceType",
        );
        let Ok(reply) = call_dbus_method(dbus, &message) else { return false };
        let Ok(device_type) = reply.try_child_get::<String>(0) else { return false };
        let Some(device_type) = device_type else { return false };

        device_type == "mouse"
    });
    let Some(mouse) = mouse else {
        tracing::debug!("no mouse found");
        REDUCER.emit(OpenRazerInput::MouseDetected(false));

        return Ok(());
    };
    tracing::trace!({ mouse }, "found mouse");

    tracing::trace!({ mouse }, "getting battery level");
    let battery = {
        let message = DBusMessage::new_method_call(
            OPENRAZER_BUS_NAME,
            &format!("/org/razer/device/{mouse}"),
            Some("razer.device.power"),
            "getBattery",
        );
        let reply = call_dbus_method(dbus, &message)?;

        reply
            .try_child_get::<f64>(0)?
            .ok_or_else(|| anyhow!("failed to get mouse battery level"))?
    };
    tracing::trace!({ battery, mouse }, "got battery level");

    REDUCER.emit(OpenRazerInput::MouseBattery(battery));

    Ok(())
}
