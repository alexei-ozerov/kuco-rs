use k8s_openapi::api::core::v1::Namespace;

use kube::ResourceExt;
use kube::{
    Client,
    api::{Api, ListParams},
};

#[derive(Clone, Debug)]
pub struct NamespaceData {
    pub names: Vec<String>,
}

impl Default for NamespaceData {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceData {
    pub fn new() -> Self {
        NamespaceData { names: Vec::new() }
    }

    pub async fn update(&mut self, client: Client) {
        let ns_api_data: Api<Namespace> = Api::all(client);

        // List all pods in the namespace.
        let lp = ListParams::default();
        let ns_list = ns_api_data.list(&lp).await.unwrap().items;

        // If a namespace was deleted, remove it as well.
        if ns_list.len() < self.names.len() {
            let mut replacement_vec: Vec<String> = Vec::new();

            // TODO: find a better way to do this ...
            for ns in &ns_list {
                let ns_name = ns.name_any();
                replacement_vec.push(ns_name);
            }
            self.names = replacement_vec;
        }

        for ns in ns_list {
            let ns_name = ns.name_any();

            // If not already in the array, add it.
            if !self.names.contains(&ns_name) {
                self.names.push(ns_name);
            }
        }
    }
}
