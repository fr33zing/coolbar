use std::collections::BTreeMap;

pub type MonitorConnector = String;
pub type WorkspaceId = usize;
pub type WindowId = usize;

pub trait WaylandCompositorMonitor {
    fn connector(&self) -> &str;
}

pub trait WaylandCompositorWorkspace {
    fn name(&self) -> &str;
}

pub trait WaylandCompositorWindow {
    fn class(&self) -> &str;
    fn title(&self) -> &str;
}

pub trait WaylandCompositor {
    type Monitor;
    type Workspace;
    type Window;

    fn monitors(&self) -> &BTreeMap<MonitorConnector, Self::Monitor>;
    fn active_monitor(&self) -> &Self::Monitor;
    fn monitor_is_empty(&self, monitor: &Self::Monitor) -> bool;

    fn workspaces(&self) -> &BTreeMap<WorkspaceId, Self::Workspace>;
    fn active_workspace(&self, monitor: &Self::Monitor) -> Option<&Self::Workspace>;
    fn workspace_is_empty(&self, workspace: &Self::Workspace) -> bool;
    fn workspaces_in_monitor(&self, monitor: &Self::Monitor) -> Vec<&Self::Workspace>;

    fn windows(&self) -> &BTreeMap<WindowId, Self::Window>;
    fn active_window(&self, workspace: &Self::Workspace) -> Option<&Self::Window>;
    fn windows_in_workspace(&self, workspace: &Self::Workspace) -> Vec<&Self::Window>;
}
