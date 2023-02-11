use k8s_openapi::{
    api::core::v1::{Pod, PodCondition},
    serde_json::{self, json},
};
use kube::{
    api::{DeleteParams, Log, LogParams, PostParams},
    Api, Client,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::log::info;

#[derive(Debug, Error)]
pub enum KubeError {
    #[error("kubers error")]
    KubeRSError {
        #[from]
        source: kube::Error,
    },
    #[error("kubers response error")]
    KubeRSResponseError {
        #[from]
        sourece: kube::error::ErrorResponse,
    },
    #[error("serde error")]
    SerdeError {
        #[from]
        source: serde_json::Error,
    },
    #[error("ok_or error")]
    OkOrError(String),
}
pub type Result<T, E = KubeError> = std::result::Result<T, E>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PodStatus {
    pub phase: String,
    pub conditions: Vec<PodCondition>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ResourceRequirements {
    limits: Option<Vec<(String, String)>>,
    requests: Option<Vec<(String, String)>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PodInfo {
    name: String,
    image: String,
    resource_requirements: Option<ResourceRequirements>,
}

pub struct PodLogReq {
    pub pod_name: String,
    pub tail_lines: i64,
}
#[derive(Clone)]
pub struct Kube(Client);

impl From<Client> for Kube {
    fn from(value: Client) -> Self {
        Self(value)
    }
}

impl Kube {
    pub async fn query_pod_status(&self, namespace: &str, pod_name: &str) -> Result<PodStatus> {
        let pods: Api<Pod> = Api::namespaced(self.0.to_owned(), namespace);
        let pod = pods.get(pod_name).await?;
        let phase = pod
            .status
            .as_ref()
            .ok_or(KubeError::OkOrError(
                "get pod status as ref fail".to_string(),
            ))?
            .phase
            .clone()
            .ok_or(KubeError::OkOrError("get pod phase clone fail".to_string()))?;
        let conditions = pod
            .status
            .as_ref()
            .ok_or(KubeError::OkOrError(
                "get pod status as ref fail".to_string(),
            ))?
            .conditions
            .as_ref()
            .ok_or(KubeError::OkOrError(
                "get pod condition as ref fail".to_string(),
            ))?;

        let pod_status = PodStatus {
            phase,
            conditions: conditions.to_vec(),
        };
        Ok(pod_status)
    }

    pub async fn query_pod_logs(&self, namespace: &str, pod_log_req: &PodLogReq) -> Result<String> {
        let pods: Api<Pod> = Api::namespaced(self.0.to_owned(), namespace);
        let mut lp = LogParams::default();
        lp.tail_lines = Some(pod_log_req.tail_lines);
        let logs = pods.logs(&pod_log_req.pod_name, &lp).await?;
        Ok(logs)
    }

    pub async fn create_pod(&self, namespace: &str, pod_info: &PodInfo) -> Result<Pod> {
        // Manage pods
        let pods: Api<Pod> = Api::namespaced(self.0.to_owned(), namespace);

        info!("Creating Pod instance");
        let p: Pod = serde_json::from_value(json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": { "name":  pod_info.name },
            "spec": {
                "containers": [{
                "name": pod_info.name,
                "image": pod_info.image
                }],
            }
        }))?;
        let pp = PostParams::default();
        let pod = pods.create(&pp, &p).await?;
        info!("Created {:?}", pod);
        Ok(p)
    }

    pub async fn stop_pod(&self, namespace: &str, pod_name: &str) -> Result<()> {
        let pods: Api<Pod> = Api::namespaced(self.0.to_owned(), namespace);
        pods.delete(pod_name, &DeleteParams::default()).await?;
        Ok(())
    }

    async fn delete_pod() {
        // let res = core.delete_namespaced_pod(pod_name, namespace, &DeleteParams::default())
        //     .await
        //     .expect("Failed to delete pod");
        // println!("Deleted pod: {:?}", res);

        // Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use axum::http::Uri;
    use kube::{
        config::{KubeConfigOptions, Kubeconfig},
        Client, Config,
    };
    use tokio::time::{interval_at, sleep, Duration};

    use crate::kube::{PodInfo, PodLogReq};

    use super::Kube;

    #[tokio::test]
    async fn create_pod() {
        // let cwd = env::current_dir().unwrap();
        // let kube_config = Kubeconfig::read_from(cwd.join("kubeconfig.yaml")).unwrap();
        // let kb = KubeConfigOptions {
        //     context: kube_config.contexts,
        //     cluster: kube_config.clusters,
        //     user: kube_config.
        // }
        let config = Config::infer().await.unwrap();
        let client = Client::try_from(config).unwrap();
        let kube = Kube::from(client);
        let pod_info = PodInfo {
            name: "tester".to_string(),
            image: "nginx".to_string(),
            resource_requirements: None,
        };
        let namespace = "default";
        match kube.query_pod_status(namespace, &pod_info.name).await {
            Ok(o) => {
                kube.stop_pod(namespace, &pod_info.name).await.unwrap();
            }
            Err(e) => {}
        }
        sleep(Duration::from_millis(3000)).await;
        kube.create_pod(namespace, &pod_info).await.unwrap();
        sleep(Duration::from_millis(10000)).await;
        let logs = kube
            .query_pod_logs(
                namespace,
                &PodLogReq {
                    pod_name: pod_info.name.clone(),
                    tail_lines: 100,
                },
            )
            .await
            .unwrap();
        println!("logs: {}", logs);
        kube.stop_pod(namespace, &pod_info.name).await.unwrap();
    }
}
