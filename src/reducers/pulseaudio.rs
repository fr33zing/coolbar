use std::{cell::RefCell, ops::Deref, rc::Rc};

use anyhow::{bail, ensure, Context as AnyhowContext, Result};
use libpulse_binding as pulse;
use pulse::{
    callbacks::ListResult,
    context::{subscribe::InterestMaskSet, Context, FlagSet, State},
    mainloop::threaded::Mainloop,
    proplist::Proplist,
};
use relm4::{Reducer, Reducible};
use tokio::{sync::OnceCell, task};

const CONTEXT_APP_NAME: &str = "FooApp"; // TODO change me lol
const CONTEXT_NAME: &str = "FooAppContext";

pub static REDUCER: Reducer<PulseAudioReducer> = Reducer::new();

static DEFAULT_SINK_NAME: OnceCell<String> = OnceCell::const_new();

#[derive(Debug, Clone)]
pub struct PulseAudioReducer {
    /// Volume of the first channel of the default sink. Value has been multiplied to represent
    /// percentage, i.e. 0-100 rather than the 0-65535 that pulse provides.
    pub volume: f32,

    /// Indicates whether the default sink is muted or not.
    pub muted: bool,
}

pub enum PulseAudioInput {
    Update(u32, bool),
}

impl Reducible for PulseAudioReducer {
    type Input = PulseAudioInput;

    fn init() -> Self {
        task::spawn(async move {
            if let Err(err) = connect().await {
                tracing::error!("pulse connection failed: {err}");
            }
        });

        Self {
            volume: 0.0,
            muted: false,
        }
    }

    fn reduce(&mut self, input: Self::Input) -> bool {
        match input {
            PulseAudioInput::Update(volume, muted) => {
                self.volume = f32::round((volume as f32) / 65535.0 * 100.0);
                self.muted = muted;
                tracing::trace!("volume: {}%, muted: {}", self.volume, muted);
            }
        }
        true
    }
}

// Adapted from the following example:
// https://docs.rs/libpulse-binding/2.26.0/libpulse_binding/mainloop/threaded/index.html
async fn connect() -> Result<()> {
    tracing::debug!("connecting to pulse server");

    let proplist = Proplist::new();
    ensure!(proplist.is_some(), "failed to create proplist");
    let mut proplist = proplist.unwrap();
    let proplist_result = proplist.set_str(
        pulse::proplist::properties::APPLICATION_NAME,
        CONTEXT_APP_NAME,
    );
    ensure!(proplist_result.is_ok(), "failed to setup proplist");

    let mainloop = Rc::new(RefCell::new(
        Mainloop::new().context("failed to create mainloop")?,
    ));
    let context = Rc::new(RefCell::new(
        Context::new_with_proplist(mainloop.borrow().deref(), CONTEXT_NAME, &proplist)
            .context("failed to create new context")?,
    ));

    // Context state change callback
    {
        let ml_ref = Rc::clone(&mainloop);
        let context_ref = Rc::clone(&context);
        context
            .borrow_mut()
            .set_state_callback(Some(Box::new(move || {
                let state = unsafe { (*context_ref.as_ptr()).get_state() };
                match state {
                    State::Ready | State::Failed | State::Terminated => unsafe {
                        (*ml_ref.as_ptr()).signal(false);
                    },
                    _ => {}
                }
            })));
    }

    context
        .borrow_mut()
        .connect(None, FlagSet::NOFLAGS, None)
        .context("failed to connect context")?;
    mainloop
        .borrow_mut()
        .start()
        .context("failed to start mainloop")?;

    // Wait for context to be ready
    loop {
        match context.borrow().get_state() {
            State::Ready => break,
            State::Failed | State::Terminated => {
                mainloop.borrow_mut().unlock();
                mainloop.borrow_mut().stop();
                bail!("context state failed/terminated");
            }
            _ => mainloop.borrow_mut().wait(),
        };
    }

    // Subscribe to sink events
    {
        let context_ref = Rc::clone(&context);
        context
            .borrow_mut()
            .set_subscribe_callback(Some(Box::new(move |_, _, _| {
                let Some(default_sink_name) = DEFAULT_SINK_NAME.get() else {
                    tracing::warn!("got sink event before default sink is known");
                    return;
                };
                unsafe {
                    (*context_ref.as_ptr()).introspect().get_sink_info_by_name(
                        &default_sink_name,
                        move |info| {
                            let ListResult::Item(info) = info else { return };
                            let volume = info.volume.get()[0].0;
                            let muted = info.mute;
                            REDUCER.emit(PulseAudioInput::Update(volume, muted));
                        },
                    )
                };
            })));

        context
            .borrow_mut()
            .subscribe(InterestMaskSet::SINK, |success| {
                if success {
                    tracing::debug!("successfully subscribed to sink events");
                } else {
                    tracing::debug!("failed to subscribe to sink events");
                }
            });
    }

    // Get default sink
    {
        let introspect = context.borrow().introspect();
        introspect.get_server_info(move |info| {
            let Some(default_sink_name) = &info.default_sink_name else {
                tracing::error!("failed to find default sink");
                return;
            };

            DEFAULT_SINK_NAME
                .set(default_sink_name.to_string())
                .expect("failed to store default sink name");
            tracing::debug!("got default sink name: {}", default_sink_name);

            // Get initial state
            let introspect = context.borrow().introspect();
            introspect.get_sink_info_by_name(default_sink_name, move |info| {
                let ListResult::Item(info) = info else { return };
                let volume = info.volume.get()[0].0;
                let muted = info.mute;
                REDUCER.emit(PulseAudioInput::Update(volume, muted));
            });
        });
    }

    Ok(())
}
