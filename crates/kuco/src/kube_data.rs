use kuco_k8s_backend::{KubeContext, NamespaceData, PodData, PodInfo};

pub struct KubeData {
    context: KubeContext,
    pub namespaces: NamespaceData,
    pub pods: PodData,
    pub current_pod: PodInfo,
}

impl KubeData {
    pub async fn new() -> Self {
        KubeData {
            context: KubeContext::default(),
            namespaces: NamespaceData::default(),
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
        self.namespaces
            .update(
                self.context
                    .client
                    .clone() // TODO: check if there is a way to avoid cloning ...
                    .expect("[ERROR] Client is None."),
            )
            .await;
    }
}
