use anyhow::Result;
use bollard::{
    container::{self, StartContainerOptions},
    image,
    secret::{ContainerInspectResponse, ImageInspect},
    Docker,
};
use futures_util::TryStreamExt;

use crate::{context::Context, presentation, services::Services};

pub async fn deploy(context: &Context, docker: &Docker) -> Result<()> {
    deploy_dependencies(context, docker).await?;

    Ok(())
}

pub async fn deploy_dependencies(context: &Context, docker: &Docker) -> Result<()> {
    let services = Services::from_context(&context);

    let container_configs = services.to_container_configs(&context)?;

    for config in container_configs {
        let container_name = config.container_name();
        let image_name = config.image_name();
        let config = config.config();

        presentation::print_dependency_pulling(container_name);
        docker
            .create_image(
                Some(image::CreateImageOptions {
                    from_image: image_name,
                    tag: "latest",
                    ..Default::default()
                }),
                None,
                None,
            )
            .try_collect::<Vec<_>>()
            .await?;

        let image_info = docker
            .inspect_image(format!("{}:latest", image_name).as_str())
            .await?;
        let existing_container = match docker.inspect_container(container_name, None).await {
            Ok(container) => Some(container),
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => None,
            Err(e) => return Err(e.into()),
        };

        if should_recreate_dependency_container(&image_info, existing_container.as_ref()) {
            presentation::print_dependency_creating(container_name);

            if existing_container.is_some() {
                docker
                    .remove_container(
                        container_name,
                        Some(container::RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await?;
            }

            docker
                .create_container(
                    Some(container::CreateContainerOptions {
                        name: container_name,
                        platform: None,
                    }),
                    config.clone(),
                )
                .await?;

            presentation::print_dependency_starting(container_name);
            docker
                .start_container(container_name, None::<StartContainerOptions<String>>)
                .await?;

            presentation::print_dependency_success(container_name);
        } else {
            presentation::print_dependency_exists(container_name);
        }
    }

    Ok(())
}

pub fn should_recreate_dependency_container(
    image_info: &ImageInspect,
    container_info: Option<&ContainerInspectResponse>,
) -> bool {
    let Some(image_id) = &image_info.id else {
        return true;
    };

    let Some(container_image) = container_info.and_then(|info| info.image.as_ref()) else {
        return true;
    };

    if container_image == image_id {
        return false;
    }

    true
}
