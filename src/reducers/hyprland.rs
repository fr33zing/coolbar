use std::{collections::HashMap, env};

use anyhow::{anyhow, bail, Result};
use relm4::{Reducer, Reducible};
use serde::Deserialize;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    task,
};
use tracing::{error, trace};

pub static REDUCER: Reducer<HyprlandReducer> = Reducer::new();

#[derive(Debug, Clone, Deserialize)]
pub struct HyprctlMonitorActiveWorkspace {
    pub id: i16,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyprctlMonitor {
    pub id: i16,
    pub name: String,
    pub focused: bool,
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: HyprctlMonitorActiveWorkspace,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyprctlWorkspace {
    pub id: i16,
    pub name: String,
    pub monitor: String,
    pub windows: i16,
}

type HyprctlMonitorsResponse = Vec<HyprctlMonitor>;
type HyprctlWorkspacesResponse = Vec<HyprctlWorkspace>;

#[derive(Debug, Clone)]
pub struct HyprlandReducer {
    pub monitors: HyprctlMonitorsResponse,
    pub workspaces: HashMap<i16, HyprctlWorkspace>,
    pub active_monitor: u16,
    pub active_workspace: i16,
    pub active_window_app: String,
    pub active_window_title: String,
}

pub enum HyprlandInput {
    Refresh,
    ActiveWorkspace(HyprctlMonitorsResponse, HyprctlWorkspacesResponse, i16),
    ActiveWindow(String, String),
    CloseWindow,
}

impl Reducible for HyprlandReducer {
    type Input = HyprlandInput;

    fn init() -> Self {
        task::spawn(async move {
            if let Err(err) = connect_event_socket().await {
                error!("hyprland event socket connection failed: {err}");
            }
        });

        REDUCER.emit(HyprlandInput::Refresh);

        Self {
            monitors: Default::default(),
            workspaces: Default::default(),
            active_monitor: 0,
            active_workspace: 0,
            active_window_app: "?".into(),
            active_window_title: "?".into(),
        }
    }

    fn reduce(&mut self, input: Self::Input) -> bool {
        match input {
            HyprlandInput::Refresh => {
                task::spawn(async move {
                    if let Err(err) = refresh().await {
                        error!("getting initial hyprland workspace state failed: {err}");
                    }
                });
            }
            HyprlandInput::ActiveWorkspace(monitors, workspaces, active_workspace) => {
                self.monitors = monitors;
                self.workspaces = workspaces
                    .iter()
                    .map(|ws| (ws.id - 1, ws.to_owned()))
                    .collect();
                self.active_workspace = active_workspace - 1;
            }
            HyprlandInput::ActiveWindow(class, title) => {
                self.active_window_app = class;
                self.active_window_title = title;

                REDUCER.emit(HyprlandInput::Refresh);
            }
            HyprlandInput::CloseWindow => {
                REDUCER.emit(HyprlandInput::Refresh);
            }
        }
        true
    }
}

async fn monitors() -> Result<HyprctlMonitorsResponse> {
    let json = send(b"[j]/monitors").await?;
    let response = serde_json::from_slice::<HyprctlMonitorsResponse>(&json)?;
    Ok(response)
}

async fn workspaces() -> Result<HyprctlWorkspacesResponse> {
    let json = send(b"[j]/workspaces").await?;
    let response = serde_json::from_slice::<HyprctlWorkspacesResponse>(&json)?;
    Ok(response)
}

async fn active_monitor() -> Result<i16> {
    // TODO ask gtk
    Ok(0)
}

async fn refresh() -> Result<()> {
    let (monitors, workspaces) = tokio::try_join!(monitors(), workspaces())?;
    let active_monitor = active_monitor().await?;
    let active_monitor = monitors.iter().find(|m| m.id == active_monitor);
    let Some(active_monitor) = active_monitor else {
        bail!("failed to determine active monitor");
    };
    let active_workspace = active_monitor.active_workspace.id;

    REDUCER.emit(HyprlandInput::ActiveWorkspace(
        monitors,
        workspaces,
        active_workspace,
    ));
    Ok(())
}

async fn update(active_workspace: i16) -> Result<()> {
    let (monitors, workspaces) = tokio::try_join!(monitors(), workspaces())?;
    REDUCER.emit(HyprlandInput::ActiveWorkspace(
        monitors,
        workspaces,
        active_workspace,
    ));
    Ok(())
}

async fn control_socket_stream() -> Result<UnixStream> {
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
    let socket = format!("/tmp/hypr/{hyprland_instance_signature}/.socket.sock");
    let stream = UnixStream::connect(socket).await?;
    Ok(stream)
}

async fn send(command: &[u8]) -> Result<Vec<u8>> {
    let mut stream = control_socket_stream().await?;
    stream.writable().await?;
    stream.write_all(command).await?;

    let mut buf = Vec::with_capacity(4096);
    loop {
        stream.readable().await?;
        match stream.read_buf(&mut buf).await {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(buf)
}

async fn connect_event_socket() -> Result<()> {
    let hyprland_instance_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE")?;
    let socket = format!("/tmp/hypr/{hyprland_instance_signature}/.socket2.sock");
    let stream = UnixStream::connect(socket).await?;
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    loop {
        let Some(line) = lines.next_line().await? else { continue };
        let malformed_err = || anyhow!("malformed hyprland socket message: {line}");
        let (key, value) = line.split_once(">>").ok_or_else(malformed_err)?;

        match key {
            "workspace" => {
                let active_workspace = value.parse::<i16>()?;
                update(active_workspace).await?;
            }
            "activewindow" => {
                let (class, title) = value.split_once(",").ok_or_else(malformed_err)?;
                trace!({ class, title }, "active window changed");
                REDUCER.emit(HyprlandInput::ActiveWindow(class.into(), title.into()));
            }
            "closewindow" => {
                REDUCER.emit(HyprlandInput::CloseWindow);
            }
            _ => {
                trace!({ event = line }, "unhandled hyprland socket event");
            }
        }
    }
}
