use std::io::{self, Read, Seek, Write};

use anyhow::Result;
use bollard::Docker;
use futures_util::StreamExt;

use crate::{services::app::AppService, utils::file::Empty};

const IGNORE_FILE: &str = ".dockerignore";
const DOCKERFILE_FILE: &str = "Dockerfile";
const ALWAYS_INCLUDE_FILES: &[&str] = &["Dockerfile", ".dockerignore"];

pub async fn build_app_service_image(app_service: &AppService, docker: &Docker) -> Result<String> {
    let bytes = create_cwd_tar()?;

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
                aux: Some(bollard::models::BuildInfoAux::Default(image_id_inner)),
                ..
            } => {
                image_id = image_id_inner.id;
            }
            log => {
                println!("{log:?}");
            }
        }
    }

    image_id.ok_or_else(|| anyhow::anyhow!("Failed to build image"))
}

fn create_cwd_tar() -> Result<Vec<u8>> {
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

    for file_name in ALWAYS_INCLUDE_FILES {
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
