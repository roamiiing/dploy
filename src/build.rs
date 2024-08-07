use std::io::{self, Read, Seek, Write};

use anyhow::Result;
use console::style;
use futures_util::StreamExt;

use crate::{context, services, utils::file::Empty};

const IGNORE_FILE: &str = ".dockerignore";

pub async fn build_app_service_image(
    context: &context::Context,
    app_service: &services::app::AppService,
    docker: &bollard::Docker,
) -> Result<String> {
    let bytes = create_cwd_tar(context)?;

    let mut stream = docker.build_image(
        app_service.to_image_build_config(),
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
    let mut bytes = Vec::<u8>::new();
    let mut archive = tar::Builder::new(&mut bytes);

    let walker = create_walker();

    for entry in walker.filter_map(Result::ok) {
        let metadata = entry.metadata()?;

        if !metadata.is_file() {
            continue;
        }

        archive.append_path(entry.path())?;
    }

    for file_name in get_always_include_files(context) {
        let _ = archive.append_path(file_name);
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

fn create_walker() -> ignore::Walk {
    let mut builder = ignore::WalkBuilder::new("./");

    builder.add_ignore(IGNORE_FILE);
    builder
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .build()
}

fn get_always_include_files(context: &context::Context) -> Vec<String> {
    let dockerfile = context.app_config().dockerfile(context.override_context());

    vec![dockerfile.to_owned(), IGNORE_FILE.to_owned()]
}
