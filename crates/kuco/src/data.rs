/*
 * Convert data from the k8s backend to structures consumed by the TUI.
 */

use ratatui::widgets::ListState;

use kuco_k8s_backend::{ContainerData, KubeContext, LogData, NamespaceData, PodData, PodInfo};

/*
 * Create a generic Kube Component State Structure.
 */

// TODO: Move this to a more appropriate location.
#[derive(Debug, Clone)]
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
            search: Search {
                input: "".to_string(),
            },
            list_state: ListState::default(),
        }
    }
}

/*
 * Aggregate Kube Data
 */

#[derive(Clone)]
pub struct KubeData {
    context: KubeContext,

    // Markers for current selection.
    pub current_namespace: Option<String>,
    pub current_pod_name: Option<String>,
    pub current_log_line: Option<String>,
    pub current_container: Option<String>,

    pub current_pod_info: PodInfo,

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
    pub async fn new() -> Self {
        KubeData {
            context: KubeContext::default(),
            namespaces: NamespaceData::new(),
            current_namespace: None,
            current_log_line: None,
            pods: PodData::default(),
            current_pod_info: PodInfo::default(),
            current_pod_name: None,
            containers: ContainerData::new(),
            current_container: None,
            logs: LogData::new(),
        }
    }

    pub fn get_namespaces(&mut self) -> Vec<String> {
        let mut ref_ns_vec = Vec::<String>::new();

        self.namespaces.names.iter().for_each(|ns| {
            ref_ns_vec.push(ns.to_string());
        });

        ref_ns_vec
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

    pub async fn update_namespaces_names_list(&mut self) {
        self.namespaces
            .update(
                self.context
                    .client
                    .clone() // TODO: check if there is a way to avoid cloning ...
                    .expect("[ERROR] Client is None."),
            )
            .await;
    }

    // Update PodData object and Pods List Vector
    pub async fn update_pods(&mut self) {
        let ns: String = match &self.current_namespace {
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
        let ns: String = match &self.current_namespace {
            Some(s) => s.to_owned(),
            None => "default".to_owned(),
        };

        match &self.current_pod_name {
            Some(po) => {
                match &self.current_container {
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
        let ns: String = match &self.current_namespace {
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
        let ns: String = match &self.current_namespace {
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
