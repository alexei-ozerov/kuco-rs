/*
 * Convert data from the k8s backend to structures consumed by the TUI.
 */

use ratatui::widgets::ListState;

use kuco_k8s_backend::{KubeContext, NamespaceData, PodData, PodInfo};

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
    pub current_namespace: Option<String>,
    pub current_pod_name: Option<String>,
    pub current_pod_info: PodInfo,
    pub current_container: Option<String>, // TODO: Create custom types ...
                                           //
    pub namespaces: NamespaceData,
    pub pods: PodData,
    pub container: Option<String>,
    pub logs: Option<String>,
}

// TODO: Replace calls to the kubeapi here with calls to the database.
//       The calls to K8s should happen continually on another thread
//       and write to the sqlite database.
impl KubeData {
    pub async fn new() -> Self {
        KubeData {
            context: KubeContext::default(),
            namespaces: NamespaceData::new(),
            current_namespace: None,
            pods: PodData::default(),
            current_pod_info: PodInfo::default(),
            current_pod_name: None,
            container: None,
            current_container: None,
            logs: None,
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
        let ref_pods_vec = self.pods.names.clone();
        // let ref_pods_vec = self
        //     .pods
        //     .list
        //     .iter()
        //     .map(|po| po.name.to_string())
        //     .collect();

        ref_pods_vec
    }

    pub async fn update_all(&mut self) {
        self.update_context().await;
        self.update_namespaces().await;
        self.update_pods_names_list().await;
    }

    pub async fn update_context(&mut self) {
        // TODO: Implement custom error types for tui to replace unwrap().
        if self.context.client.is_none() {
            self.context.init_context().await.unwrap();
        }
    }

    pub async fn update_namespaces(&mut self) {
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
        let ns: String;

        match &self.current_namespace {
            Some(s) => ns = s.to_owned(),
            None => ns = "default".to_owned(),
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

    pub async fn update_pods_names_list(&mut self) {
        let ns: String;

        match &self.current_namespace {
            Some(s) => ns = s.to_owned(),
            None => ns = "default".to_owned(),
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
