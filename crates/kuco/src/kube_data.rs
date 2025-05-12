/*
 * Convert data from the k8s backend to structures consumed by the TUI.
 */

use ratatui::widgets::ListState;

use kuco_k8s_backend::{KubeContext, NamespaceData, PodData, PodInfo};

/*
 * Namespace Data
 */
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

// TODO: Move this to a more appropriate location.
#[derive(Debug, Clone)]
pub struct Search {
    pub input: String,
}

pub struct NamespaceListState {
    pub ns_list_state: ListState,
    pub search: Search,
}

impl NamespaceListState {
    fn new() -> Self {
        NamespaceListState {
            search: Search {
                input: "".to_string(),
            },
            ns_list_state: ListState::default(),
        }
    }

    // TODO: Input should name itself after cluster context or something.
    // pub fn build_input(&self) -> Paragraph {
    //     /// Max width of the UI box showing current mode
    //     const MAX_WIDTH: usize = 14;
    //     let (pref, mode) = (" ", "GLOBAL");
    //     let mode_width = MAX_WIDTH - pref.len();
    //     let input = format!("[{pref}{mode:^mode_width$}] {}", self.search.input.as_str(),);
    //     let input = Paragraph::new(input);
    //
    //     input.block(
    //         Block::default()
    //             .borders(Borders::LEFT | Borders::RIGHT)
    //             .border_type(BorderType::Rounded)
    //             .title(format!("{:─>width$}", "", width = 12)),
    //     )
    // }
}

/*
 * Aggregate Kube Data
 */

pub struct KubeData {
    context: KubeContext,
    pub namespace_list: NamespaceList,
    pub pods: PodData,
    pub current_pod: PodInfo,
}

impl KubeData {
    pub async fn new() -> Self {
        KubeData {
            context: KubeContext::default(),
            namespace_list: NamespaceList::new(),
            pods: PodData::default(),
            current_pod: PodInfo::default(),
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
        self.namespace_list.namespaces
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
    pub namespace_state: NamespaceListState,
}

impl KubeState {
    pub fn new() -> Self {
        Self {
            namespace_state: NamespaceListState::new()
        }
    }
}
