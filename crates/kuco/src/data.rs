/*
 * Convert data from the k8s backend to structures consumed by the TUI.
 */

use color_eyre::{Result, eyre::WrapErr};
use ratatui::widgets::ListState;
use std::sync::Arc;

use kuco_k8s_backend::{
    containers::ContainerData,
    context::KubeContext,
    logs::LogData,
    namespaces::NamespaceData,
    pods::{PodData, PodInfo},
};
use kuco_sqlite_backend::{KucoSqliteStore, SqliteCache};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NamespaceList {
    namespaces: Vec<String>,
}

/*
 * Create a generic Kube Component State Structure.
 */

#[derive(Debug, Clone, Default)]
pub struct Search {
    pub input: String,
}

#[derive(Clone, Debug)]
pub struct KubeComponentState {
    pub list_state: ListState,
    pub search: Search,
}

impl KubeComponentState {
    fn new() -> Self {
        KubeComponentState {
            search: Search::default(),
            list_state: ListState::default(),
        }
    }
}

/*
 * Aggregate Kube Data
 */

#[derive(Clone)]
pub struct KubeData {
    arc_ctx: Arc<SqliteCache>,
    context: KubeContext,

    // Markers for current selection.
    pub current_namespace_name: Option<String>,
    pub current_pod_name: Option<String>,
    pub current_container_name: Option<String>,
    pub current_log_line: Option<String>,

    pub current_pod_info: PodInfo,

    // TODO: Refactor old components into new ones from cache
    pub namespace_name_list: Vec<String>,

    // TODO: Refactor old struct members
    pub namespaces: NamespaceData,
    pub pods: PodData,
    pub containers: ContainerData,
    pub logs: LogData,
}

// TODO: Why do you use default() sometimes and new() other times ... standarize please
// TODO: Replace calls to the kubeapi here with calls to the database.
//       The calls to K8s should happen continually on another thread
//       and write to the sqlite database.
impl KubeData {
    pub async fn new(arc_ctx: Arc<SqliteCache>) -> Self {
        KubeData {
            arc_ctx,
            context: KubeContext::default(),
            namespaces: NamespaceData::new(),
            current_namespace_name: None,
            current_log_line: None,
            pods: PodData::default(),
            current_pod_info: PodInfo::default(),
            current_pod_name: None,
            containers: ContainerData::new(),
            current_container_name: None,
            logs: LogData::new(),
            namespace_name_list: Vec::new(),
        }
    }

    pub fn get_namespaces(&mut self) -> Vec<String> {
        // let ref_ns_vec = self
        //     .namespaces
        //     .names
        //     .iter()
        //     .map(|ns| ns.to_string())
        //     .collect();
        //
        // ref_ns_vec

        self.namespace_name_list.clone()
    }

    pub fn get_pods(&mut self) -> Vec<String> {
        self.pods.names.clone()
    }

    pub fn get_logs(&mut self) -> Vec<String> {
        self.logs.lines.clone()
    }

    pub fn get_containers(&mut self) -> Vec<String> {
        self.containers.names.clone()
    }

    pub async fn update_all(&mut self) {
        self.update_context().await;
        self.update_namespaces_names_list().await;
        self.update_pods_names_list().await;
    }

    pub async fn update_context(&mut self) {
        // TODO: Implement custom error types for tui to replace unwrap().
        if self.context.client.is_none() {
            self.context.init_context().await.unwrap();
        }
    }

    pub async fn update_namespaces_names_list(&mut self) -> Result<()> {
        let store = &self.arc_ctx;

        let table_name = "kv_cache".to_owned();
        let key_name = "all_namespaces".to_owned();

        let fetched_namespaces: Vec<String> = store
            .get_json::<Vec<String>>(table_name, key_name.clone())
            .await
            .wrap_err_with(|| format!("Failed to get JSON for key '{}'", key_name.clone()))?
            .unwrap_or_default();

        self.namespace_name_list = fetched_namespaces;

        Ok(())
    }

    // Update PodData object and Pods List Vector
    pub async fn update_pods(&mut self) {
        let ns: String = match &self.current_namespace_name {
            Some(s) => s.to_owned(),
            None => "default".to_owned(),
        };

        let _ = self
            .pods
            .update_all(
                self.context
                    .client
                    .clone() // TODO: check if there is a way to avoid cloning ...
                    .expect("[ERROR] Client is None."),
                &ns,
            )
            .await;

        self.pods.names = self.get_pods();
    }

    pub async fn update_logs_lines_list(&mut self) {
        let ns: String = match &self.current_namespace_name {
            Some(s) => s.to_owned(),
            None => "default".to_owned(),
        };

        match &self.current_pod_name {
            Some(po) => {
                match &self.current_container_name {
                    Some(co) => {
                        let _ = self
                            .logs
                            .update(
                                self.context
                                    .client
                                    .clone() // TODO: check if there is a way to avoid cloning ...
                                    .expect("[ERROR] Client is None."),
                                &ns,
                                po,
                                co,
                            )
                            .await;

                        self.containers.names = self.get_logs();
                    }
                    None => {
                        tracing::warn!(
                            "No current container selected. Nothing to do. Could be a potential bug. ;)"
                        );
                    }
                }
            }
            None => {
                tracing::warn!(
                    "No current pod selected. Nothing to do. Could be a potential bug. ;)"
                );
            }
        };
    }

    pub async fn update_containers_names_list(&mut self) {
        let ns: String = match &self.current_namespace_name {
            Some(s) => s.to_owned(),
            None => "default".to_owned(),
        };

        match &self.current_pod_name {
            Some(po) => {
                let _ = self
                    .containers
                    .update(
                        self.context
                            .client
                            .clone() // TODO: check if there is a way to avoid cloning ...
                            .expect("[ERROR] Client is None."),
                        &ns,
                        po,
                    )
                    .await;

                self.containers.names = self.get_containers();
            }
            None => {
                tracing::warn!(
                    "No current pod selected. Nothing to do. Could be a potential bug. ;)"
                );
            }
        };
    }

    pub async fn update_pods_names_list(&mut self) {
        let ns: String = match &self.current_namespace_name {
            Some(s) => s.to_owned(),
            None => "default".to_owned(),
        };

        let _ = self
            .pods
            .get_names(
                self.context
                    .client
                    .clone() // TODO: check if there is a way to avoid cloning ...
                    .expect("[ERROR] Client is None."),
                &ns,
            )
            .await;
    }
}

#[derive(Debug)]
pub struct KubeWidgetState {
    pub namespace_state: KubeComponentState,
    pub pods_state: KubeComponentState,
    pub containers_state: KubeComponentState,
    pub logs_state: KubeComponentState,
}

impl Default for KubeWidgetState {
    fn default() -> Self {
        Self::new()
    }
}

impl KubeWidgetState {
    pub fn new() -> Self {
        Self {
            namespace_state: KubeComponentState::new(),
            pods_state: KubeComponentState::new(),
            containers_state: KubeComponentState::new(),
            logs_state: KubeComponentState::new(),
        }
    }
}
