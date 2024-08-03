use anyhow::Result;
use bollard::Docker;

const CONFLICT_STATUS_CODE: u16 = 409;

pub const DPLOY_NETWORK: &str = "dploy_default";

pub async fn create_dploy_network(docker: &Docker) -> Result<()> {
    let result = docker
        .create_network(bollard::network::CreateNetworkOptions {
            name: DPLOY_NETWORK,
            ..Default::default()
        })
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: CONFLICT_STATUS_CODE,
            ..
        }) => Ok(()),
        Err(error) => Err(error.into()),
    }
}
