use std::{collections::BTreeMap, env};

use anyhow::{anyhow, Result};
use relm4::{Reducer, Reducible};
use serde::Deserialize;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    task,
};
use tracing::{error, trace};

use crate::data::wayland_compositor::{
    MonitorConnector, WaylandCompositor, WaylandCompositorMonitor, WaylandCompositorWindow,
    WaylandCompositorWorkspace, WindowId, WorkspaceId,
};

pub static REDUCER: Reducer<HyprlandReducer> = Reducer::new();

#[derive(Debug, Clone, Deserialize)]
pub struct HyprlandWrappedWorkspaceId {
    // Use isize because Hyprland uses ID -99 for the special workspace.
    pub id: isize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawHyprlandMonitor {
    #[serde(rename = "name")]
    pub connector: String,

    #[serde(rename = "focused")]
    pub active: bool,

    #[serde(rename = "activeWorkspace")]
    pub active_workspace: HyprlandWrappedWorkspaceId,
}

impl RawHyprlandMonitor {
    fn postprocess(raw: Vec<RawHyprlandMonitor>) -> BTreeMap<MonitorConnector, HyprlandMonitor> {
        let mut monitors = BTreeMap::new();

        for raw_monitor in raw {
            let connector = raw_monitor.connector;
            let processed_monitor = HyprlandMonitor {
                connector: connector.clone(),
                active: raw_monitor.active,
                active_workspace_id: RawHyprlandWorkspace::fix_id(raw_monitor.active_workspace.id),
            };
            monitors.insert(connector, processed_monitor);
        }

        monitors
    }
}

#[derive(Debug, Clone)]
pub struct HyprlandMonitor {
    pub connector: MonitorConnector,
    pub active: bool,
    pub active_workspace_id: WorkspaceId,
}

impl WaylandCompositorMonitor for HyprlandMonitor {
    fn connector(&self) -> &str {
        &self.connector
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawHyprlandWorkspace {
    pub id: isize,

    pub name: String,

    #[serde(rename = "monitor")]
    pub monitor_connector: String,

    #[serde(rename = "lastwindow")]
    pub active_window_id: String,
}

impl RawHyprlandWorkspace {
    /// Converts a 1-based isize to a 0-based usize
    fn fix_id(one_based_id: isize) -> usize {
        (one_based_id as usize) - 1
    }

    fn postprocess(raw: Vec<RawHyprlandWorkspace>) -> BTreeMap<WorkspaceId, HyprlandWorkspace> {
        let mut workspaces = BTreeMap::new();

        for raw_workspace in raw {
            // Filter special workspace
            if raw_workspace.id.is_negative() {
                continue;
            }

            let processed_workspace = HyprlandWorkspace {
                id: Self::fix_id(raw_workspace.id),
                name: raw_workspace.name,
                monitor_connector: raw_workspace.monitor_connector,
                active_window_id: RawHyprlandWindow::fix_id_prefixed(
                    raw_workspace.active_window_id,
                ),
            };

            workspaces.insert(processed_workspace.id, processed_workspace);
        }

        workspaces
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyprlandWorkspace {
    pub id: WorkspaceId,
    pub name: String,
    pub monitor_connector: String,
    pub active_window_id: WindowId,
}

impl WaylandCompositorWorkspace for HyprlandWorkspace {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawHyprlandWindow {
    #[serde(rename = "address")]
    pub id: String,
    pub class: String,
    pub title: String,
    pub workspace: HyprlandWrappedWorkspaceId,
}

impl RawHyprlandWindow {
    fn fix_id_prefixed(prefixed_hex_id: String) -> WindowId {
        usize::from_str_radix(&prefixed_hex_id[2..], 16).expect("failed to parse window id hex")
    }

    fn fix_id(hex_id: String) -> WindowId {
        usize::from_str_radix(&hex_id, 16).expect("failed to parse window id hex")
    }

    fn postprocess(raw: Vec<RawHyprlandWindow>) -> BTreeMap<WindowId, HyprlandWindow> {
        let mut windows = BTreeMap::new();

        for raw_window in raw {
            // Filter windows on special workspace
            if raw_window.workspace.id.is_negative() {
                continue;
            }

            let processed_window = HyprlandWindow {
                id: Self::fix_id_prefixed(raw_window.id),
                class: raw_window.class,
                title: raw_window.title,
                workspace_id: RawHyprlandWorkspace::fix_id(raw_window.workspace.id),
            };

            windows.insert(processed_window.id, processed_window);
        }

        windows
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct HyprlandWindow {
    pub id: WindowId,
    pub class: String,
    pub title: String,
    pub workspace_id: WorkspaceId,
}

impl WaylandCompositorWindow for HyprlandWindow {
    fn class(&self) -> &str {
        &self.class
    }

    fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Debug, Clone, Default)]
pub struct HyprlandReducer {
    initialized: bool,
    monitors: BTreeMap<MonitorConnector, HyprlandMonitor>,
    workspaces: BTreeMap<WorkspaceId, HyprlandWorkspace>,
    windows: BTreeMap<WindowId, HyprlandWindow>,
    active_monitor_connector: MonitorConnector,
    active_workspace_id: WorkspaceId,
    active_window_ids: BTreeMap<WorkspaceId, WindowId>,
}

impl WaylandCompositor for HyprlandReducer {
    type Monitor = HyprlandMonitor;
    type Workspace = HyprlandWorkspace;
    type Window = HyprlandWindow;

    fn monitors(&self) -> &BTreeMap<MonitorConnector, HyprlandMonitor> {
        &self.monitors
    }

    fn active_monitor(&self) -> &Self::Monitor {
        self.monitors
            .get(&self.active_monitor_connector)
            .expect("active monitor not found")
    }

    fn monitor_is_empty(&self, _monitor: &Self::Monitor) -> bool {
        false // TODO verify correctness
    }

    fn workspaces(&self) -> &BTreeMap<usize, Self::Workspace> {
        &self.workspaces
    }

    fn active_workspace(&self, monitor: &Self::Monitor) -> Option<&Self::Workspace> {
        self.workspaces.get(&monitor.active_workspace_id)
    }

    fn workspace_is_empty(&self, workspace: &Self::Workspace) -> bool {
        self.windows
            .values()
            .filter(|w| w.workspace_id == workspace.id)
            .count()
            == 0
    }

    fn workspaces_in_monitor(&self, monitor: &Self::Monitor) -> Vec<&Self::Workspace> {
        self.workspaces
            .values()
            .filter(|ws| ws.monitor_connector == monitor.connector)
            .collect()
    }

    fn windows(&self) -> &BTreeMap<usize, Self::Window> {
        &self.windows
    }

    fn active_window(&self, workspace: &Self::Workspace) -> Option<&Self::Window> {
        self.windows.get(&workspace.active_window_id)
    }

    fn windows_in_workspace(&self, workspace: &Self::Workspace) -> Vec<&Self::Window> {
        self.windows
            .values()
            .filter(|w| w.workspace_id == workspace.id)
            .collect()
    }
}

pub enum HyprlandInput {
    RequestRefresh,
    Refresh(
        BTreeMap<MonitorConnector, HyprlandMonitor>,
        BTreeMap<WorkspaceId, HyprlandWorkspace>,
        BTreeMap<WindowId, HyprlandWindow>,
    ),
    CloseWindow(WindowId),
    ActiveWindow(usize),
}

impl Reducible for HyprlandReducer {
    type Input = HyprlandInput;

    fn init() -> Self {
        task::spawn(async move {
            if let Err(err) = connect_event_socket().await {
                error!("hyprland event socket connection failed: {err}");
            }
        });
        REDUCER.emit(HyprlandInput::RequestRefresh);
        Self::default()
    }

    fn reduce(&mut self, input: Self::Input) -> bool {
        match input {
            HyprlandInput::RequestRefresh => {
                task::spawn(async move {
                    if let Err(err) = refresh().await {
                        error!("getting initial hyprland workspace state failed: {err}");
                    }
                });
            }
            HyprlandInput::Refresh(monitors, workspaces, windows) => {
                self.monitors = monitors;
                self.workspaces = workspaces;
                self.windows = windows;
                self.active_monitor_connector = self
                    .monitors
                    .values()
                    .find(|m| m.active)
                    .expect("no active monitor found")
                    .connector
                    .clone();
                self.active_workspace_id = self.active_monitor().active_workspace_id;
                self.active_window_ids = self
                    .workspaces
                    .values()
                    .map(|ws| (ws.id, ws.active_window_id.clone()))
                    .collect();
                self.initialized = true;
            }
            HyprlandInput::ActiveWindow(window_id) if self.initialized => {
                self.active_window_ids
                    .insert(self.active_workspace_id, window_id);
            }
            HyprlandInput::CloseWindow(window_id) if self.initialized => {
                self.windows.remove(&window_id);
            }

            _ => {}
        }
        true
    }
}

async fn refresh() -> Result<()> {
    let monitors = send(b"[j]/monitors").await?;
    let monitors: Vec<RawHyprlandMonitor> = serde_json::from_slice(&monitors)?;
    let monitors = RawHyprlandMonitor::postprocess(monitors);

    let workspaces = send(b"[j]/workspaces").await?;
    let workspaces: Vec<RawHyprlandWorkspace> = serde_json::from_slice(&workspaces)?;
    let workspaces = RawHyprlandWorkspace::postprocess(workspaces);

    let windows = send(b"[j]/clients").await?;
    let windows: Vec<RawHyprlandWindow> = serde_json::from_slice(&windows)?;
    let windows = RawHyprlandWindow::postprocess(windows);

    REDUCER.emit(HyprlandInput::Refresh(monitors, workspaces, windows));
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
            "workspace" | "openwindow" | "movewindow" => {
                refresh().await?;
            }
            "activewindowv2" if value != "," => {
                let window_id = RawHyprlandWindow::fix_id(value.to_owned());
                REDUCER.emit(HyprlandInput::ActiveWindow(window_id));
            }
            "closewindow" => {
                let window_id = RawHyprlandWindow::fix_id(value.to_owned());
                REDUCER.emit(HyprlandInput::CloseWindow(window_id));
            }
            _ => {
                trace!({key, value}, "unhandled hyprland socket event");
            }
        }
    }
}
