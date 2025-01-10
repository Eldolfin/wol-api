use crate::config;
use crate::misc::dirs;
use anyhow::Context as _;
use image::imageops::FilterType;
use regex::Regex;
use sha2::{Digest as _, Sha256};
use std::{convert::Infallible, fs, path::Path};
use tokio::io::AsyncWriteExt as _;
use tokio::{fs::File, io::AsyncReadExt as _};
use tungstenite::ClientRequestBuilder;
use utoipa::OpenApi;
use warp::{
    http,
    reject::Rejection,
    reply::{self, Reply},
    Filter,
};

const CACHE_SUBFOLDER: &str = "images";
pub const IMAGE_SIZE: u32 = 128;

/// Converts a URL to a filename-safe format with a slug and hash.
fn url_to_filename(url: &str) -> String {
    // Extract the filename from the URL using the path and extension
    let path = Path::new(url);
    let original_filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("default.txt");
    let extension = Path::new(original_filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png");
    let base_name = Path::new(original_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("default");

    // Ensure the base name is filesystem-safe
    let re = Regex::new("[^a-zA-Z0-9]+").unwrap();
    let safe_base_name = re.replace_all(base_name, "_");

    // Generate a hash of the full URL for uniqueness
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    let hash_part = format!("{hash:x}")[..8].to_string();

    // Combine the slug, hash, and extension
    format!("{safe_base_name}_{hash_part}.{extension}")
}

// TODO: avoid repetition ?
async fn cache_image_from_web(url: &str) -> anyhow::Result<String> {
    let cache_dir = dirs.cache_dir().join(CACHE_SUBFOLDER);
    fs::create_dir_all(&cache_dir)?;
    let key = url_to_filename(url);
    let filename = cache_dir.join(&key);
    let resized_filename_key = format!("{key}_resize{IMAGE_SIZE}x{IMAGE_SIZE}.png");
    let resized_filename = cache_dir.join(&resized_filename_key);

    if !resized_filename.exists() {
        let resp = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Safari/605.1.1")
            .build()?
            .get(url)
            .send()
            .await?
            .error_for_status()
            .with_context(|| format!("Failed to fetch image at {url}"))?;
        let bytes = resp.bytes().await?;
        File::create(&filename)
            .await
            .with_context(|| format!("Failed to create image file {}", filename.display()))?
            .write_all(&bytes)
            .await
            .with_context(|| format!("Failed to write image to {}", filename.display()))?;
        let image = image::load_from_memory(&bytes)
            .with_context(|| format!("Failed to load image {}", filename.display()))?
            .resize(IMAGE_SIZE, IMAGE_SIZE, FilterType::CatmullRom);
        image
            .save(&resized_filename)
            .context("Failed to write the resized image")?;
    }
    Ok(format!("/api/cache/images/{resized_filename_key}"))
}

pub fn cache_image(name: &str, icon: image::DynamicImage) -> anyhow::Result<String> {
    let cache_dir = dirs.cache_dir().join(CACHE_SUBFOLDER);
    fs::create_dir_all(&cache_dir)?;
    let key = url_to_filename(name);
    let resized_filename_key = format!("{key}_resize{IMAGE_SIZE}x{IMAGE_SIZE}.png");
    let resized_filename = cache_dir.join(&resized_filename_key);

    if !resized_filename.exists() {
        let image = icon.resize(IMAGE_SIZE, IMAGE_SIZE, FilterType::CatmullRom);
        image
            .save(&resized_filename)
            .context("Failed to write the resized image")?;
    }
    Ok(format!("/api/cache/images/{resized_filename_key}"))
}

#[expect(
    clippy::module_name_repetitions,
    reason = "it would be confusing otherwise i think"
)]
pub async fn cache_images_from_web(mut config: config::Config) -> anyhow::Result<config::Config> {
    for machine in config.machines.values_mut() {
        for task in &mut machine.tasks {
            task.icon_url = cache_image_from_web(&task.icon_url).await?;
        }
    }
    Ok(config)
}

#[utoipa::path(
    post,
    path = "/cache/images/{filename}",
    responses(
        (status = 200, description = "Image data"),
    ),
    params(
        ("filename" = String, Path, description = "Filename provided in the config")
    ),
)]
pub async fn images(filename: String) -> Result<Box<dyn Reply>, Infallible> {
    let filename = dirs.cache_dir().join(CACHE_SUBFOLDER).join(filename);

    match File::open(filename).await {
        Ok(mut file) => {
            let mut buf = Vec::new();
            match file.read_to_end(&mut buf).await {
                Ok(_) => Ok(Box::new(reply::with_status(buf, http::StatusCode::OK))),
                Err(err) => Ok(Box::new(reply::with_status(
                    format!("Could not read image: {err}"),
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                ))),
            }
        }
        Err(err) => Ok(Box::new(reply::with_status(
            format!("Image does not exist in the cache {err}"),
            http::StatusCode::NOT_FOUND,
        ))),
    }
}

pub fn image_api() -> anyhow::Result<impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone>
{
    let images =
        warp::path!("cache" / "images" / String).and_then(move |filename: String| images(filename));
    let routes = images;
    Ok(routes)
}

#[derive(OpenApi)]
#[openapi(paths(images))]
pub struct ImageApi;

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("https://example-site.com/image.png", "image_63947df0.png")]
    #[case(
        "https://example-site.com/some/path/to/file-with-no-extension",
        "file_with_no_extension_8beae082.png"
    )]
    #[case("https://example-site.com/", "example_site_b7effb5b.com")]
    #[case("/home/example/filesystem/file.png", "file_136aee85.png")]
    fn test_url_to_filename(#[case] url: &str, #[case] expected_filename: &str) {
        assert_eq!(&url_to_filename(url), expected_filename);
    }
}
