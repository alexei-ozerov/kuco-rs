/*
 * Convert data from the k8s backend to structures consumed by the TUI.
 */

use ratatui::widgets::ListState;

use kuco_k8s_backend::{KubeContext, KucoBackendError, NamespaceData, PodData, PodInfo};

/*
 * Namespace Data
 */

// Encapsulate the data coming out of `kuco_k8s_backend` in our own types.
#[derive(Clone)]
pub struct NamespaceList {
    pub namespaces: NamespaceData,
}

impl NamespaceList {
    pub fn new() -> Self {
        Self {
            namespaces: NamespaceData::new(),
        }
    }
}

// Create a generic Kube Component State Structure.
// TODO: Move this to a more appropriate location.
#[derive(Debug, Clone)]
pub struct Search {
    pub input: String,
}

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

pub struct KubeData {
    context: KubeContext,
    pub namespaces: NamespaceList,
    pub current_namespace: Option<String>,
    pub pods: PodData,
    pub current_pod: PodInfo,
    pub container: Option<String>,
    pub current_container: Option<String>, // TODO: Create custom types ...
}

impl KubeData {
    pub async fn new() -> Self {
        KubeData {
            context: KubeContext::default(),
            namespaces: NamespaceList::new(),
            current_namespace: None,
            pods: PodData::default(),
            current_pod: PodInfo::default(),
            container: None,
            current_container: None,
        }
    }

    pub async fn update_all(&mut self) {
        self.update_context().await;
        self.update_namespaces().await;
    }

    async fn update_context(&mut self) {
        // TODO: Implement custom error types for tui to replace unwrap().
        if self.context.client.is_none() {
            self.context.init_context().await.unwrap();
        }
    }

    async fn update_namespaces(&mut self) {
        self.namespaces.namespaces
            .update(
                self.context
                    .client
                    .clone() // TODO: check if there is a way to avoid cloning ...
                    .expect("[ERROR] Client is None."),
            )
            .await;
    }
}

pub struct KubeState {
    pub namespace_state: KubeComponentState,
    pub pods_state: KubeComponentState,
    pub containers_state: KubeComponentState,
    pub logs_state: KubeComponentState,
}

impl KubeState {
    pub fn new() -> Self {
        Self {
            namespace_state: KubeComponentState::new(),
            pods_state: KubeComponentState::new(),
            containers_state: KubeComponentState::new(),
            logs_state: KubeComponentState::new(),
        }
    }
}
