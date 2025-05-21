use k8s_openapi::api::core::v1::Pod;

use kube::api::LogParams;
use kube::{
    Client,
    api::Api,
};


use crate::error::KucoBackendError;

#[derive(Clone, Debug)]
pub struct LogData {
    pub lines: Vec<String>,
}

impl Default for LogData {
    fn default() -> Self {
        Self::new()
    }
}

impl LogData {
    pub fn new() -> Self {
        LogData { lines: Vec::new() }
    }

    pub async fn update(
        &mut self,
        client: Client,
        namespace: &str,
        pod_name: &str,
        container_name: &str,
    ) -> Result<(), KucoBackendError> {
        let pods_api: Api<Pod> = Api::namespaced(client, namespace);

        // 2. Define LogParams to specify the container and other log options.
        //    - `container`: Specifies which container's logs to fetch.
        //    - `follow`: If true, streams logs. Default is false (get current logs).
        //    - `timestamps`: If true, adds timestamps to log lines.
        //    - `tail_lines`: Fetches only the last N lines.
        //    - `previous`: If true, fetches logs from a previous, terminated instance of the container.
        let log_params = LogParams {
            container: Some(container_name.to_string()),
            timestamps: true, // Example: include timestamps
            // tail_lines: Some(100), // Example: get last 100 lines
            ..Default::default() // Uses default for follow (false), previous (false), etc.
        };

        let log_string = pods_api.logs(pod_name, &log_params).await.unwrap_or({
            format!(
                "Failed to fetch logs for container '{}' in pod '{}', namespace '{}'",
                container_name, pod_name, namespace
            )
        });

        let logs_vector: Vec<String> = log_string.lines().map(String::from).collect();

        self.lines = logs_vector;

        Ok(())
    }
}
