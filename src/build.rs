use std::{
    io::{self, Read, Seek, Write},
    path,
};

use anyhow::Result;
use console::style;
use futures_util::StreamExt;

use crate::{context, services, utils::file::Empty};

pub async fn build_app_service_image(
    context: &context::Context,
    app_service: &services::app::AppService,
    docker: &bollard::Docker,
) -> Result<String> {
    let bytes = create_cwd_tar(context)?;

    let mut stream = docker.build_image(
        app_service.to_image_build_config()?,
        None,
        Some(bytes.into()),
    );

    let mut image_id = None;

    while let Some(info) = stream.next().await {
        // TODO: log info from build image
        match info? {
            bollard::models::BuildInfo {
                aux: Some(image_id_inner),
                ..
            } => {
                image_id = image_id_inner.id;
            }
            bollard::models::BuildInfo {
                stream: Some(stream),
                ..
            } if !stream.trim().is_empty() => {
                let formatted_stream = if stream.ends_with('\n') {
                    stream.clone()
                } else {
                    format!("{}\n", stream)
                };

                print!("{}", style(formatted_stream).dim());
            }
            _ => {}
        }
    }

    image_id.ok_or_else(|| anyhow::anyhow!("Failed to build image"))
}

fn create_cwd_tar(context: &context::Context) -> Result<Vec<u8>> {
    let docker_context = context.app_config().context(context.override_context());

    let mut bytes = Vec::<u8>::new();
    let mut archive = tar::Builder::new(&mut bytes);

    let walker = create_walker(context);

    for entry in walker.filter_map(Result::ok) {
        let metadata = entry.metadata()?;

        if !metadata.is_file() {
            continue;
        }

        let entry_path = entry.path();
        let stripped_path = entry_path.strip_prefix(docker_context)?;

        archive.append_path_with_name(entry_path, stripped_path)?;
    }

    for file_name in get_always_include_files(context) {
        let relative_path = context
            .config_dir_relative_to_docker_context()
            .join(&file_name);

        let entry_path = path::PathBuf::from(relative_path);
        println!("Entry path: {}", entry_path.display());
        let stripped_path = entry_path.strip_prefix(docker_context)?;
        println!("Stripped path: {}", stripped_path.display());

        archive.append_path_with_name(&entry_path, stripped_path)?;
    }

    archive.into_inner()?;

    let mut compressed_file = tempfile::tempfile()?;
    let mut encoder =
        flate2::write::GzEncoder::new(&mut compressed_file, flate2::Compression::default());

    // TODO: create initial tar file and compress it by reading it in chunks
    encoder.write_all(&bytes)?;
    encoder.finish()?;

    drop(bytes);

    let mut compressed_bytes = Vec::new();
    compressed_file.seek(io::SeekFrom::Start(0))?;
    compressed_file.read_to_end(&mut compressed_bytes)?;

    let _ = compressed_file.empty();

    Ok(compressed_bytes)
}

fn create_walker(context: &context::Context) -> ignore::Walk {
    let docker_context = context.app_config().context(context.override_context());
    let mut builder = ignore::WalkBuilder::new(docker_context);

    for ignore_file in context
        .app_config()
        .ignore_files(context.override_context())
    {
        builder.add_ignore(ignore_file);
    }

    builder
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .build()
}

fn get_always_include_files(context: &context::Context) -> Vec<String> {
    let dockerfile = context.app_config().dockerfile(context.override_context());
    let ignore_files = context
        .app_config()
        .ignore_files(context.override_context());

    let mut files = vec![dockerfile.to_string()];

    for ignore_file in ignore_files {
        files.push(ignore_file.to_string());
    }

    files
}
