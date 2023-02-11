# Axum-KubeRS
Example showing how to integrate k8s with axum and kube-rs.

## Setup
- installed minikube.
- installed kubectl.
- proxy 8443 port, `kubectl proxy --port=8443 --address='0.0.0.0' --accept-hosts='.*'`.
-`kubectl config view --minify --flatten > kubeconfig.yaml`,  download it to the source root directory.
- run app `KUBECONFIG=$(pwd)/kubeconfig.yaml cargo run`.

## troubleshooting
docker mirror:  
```sh
minikube start --image-mirror-country=cn --registry-mirror="https://registry.docker-cn.com,https://docker.mirrors.ustc.edu.cn"
```

installing minikube:  
export PATH=~/.local/bin:$PATH


runtest:  
KUBECONFIG=$(pwd)/kubeconfig.yaml cargo test -- --nocapture

minikube error:  
https://forums.docker.com/t/exiting-due-to-drv-as-root-the-docker-driver-should-not-be-used-with-root-privileges/113328/2

