use anyhow::Context as _;
use image::{imageops::FilterType, DynamicImage};
use itertools::Itertools as _;
use log::debug;
use tokio::fs;

use crate::{
    cache::{download_image, searxng_api, IMAGE_SIZE},
    misc::dirs,
    utils::comparable_floats::ComparableFloats,
};

use super::{searxng_api::SearchResult, url_to_filename, CACHE_SUBFOLDER};

pub async fn cache_find_icon(application_name: &str) -> anyhow::Result<String> {
    let cache_dir = dirs.cache_dir().join(CACHE_SUBFOLDER);
    fs::create_dir_all(&cache_dir).await?;
    let key = url_to_filename(application_name);
    let resized_filename_key = format!("{key}_resize{IMAGE_SIZE}x{IMAGE_SIZE}.png");
    let resized_filename = cache_dir.join(&resized_filename_key);

    if !resized_filename.exists() {
        let image = find_icon(application_name)
            .await
            .with_context(|| {
                format!("Failed to find an icon for application `{application_name}`")
            })?
            .resize(IMAGE_SIZE, IMAGE_SIZE, FilterType::CatmullRom);
        image
            .save(&resized_filename)
            .context("Failed to write the resized image")?;
    }
    Ok(format!("/api/cache/images/{resized_filename_key}"))
}

struct IconMetadata {
    pub search_result: SearchResult,
    pub score: f32,
}

impl From<SearchResult> for IconMetadata {
    fn from(search_result: SearchResult) -> Self {
        let score = calculate_icon_score(&search_result);
        Self {
            search_result,
            score,
        }
    }
}

/// From the most important to the less important
///   - resolution
///     - it should be close enough to the expected icon size (but larger)
///     - it should be as square as possible
///   - format
///     - it should be png or svg
///   - search engine rank
fn calculate_icon_score(search_result: &SearchResult) -> f32 {
    const MIN: f32 = -1.0;

    const SQUARINESS_WEIGHT: f32 = 10000.0;
    const SEARCH_ENGINE_RANK_WEIGHT: f32 = 10000.0;
    const FORMAT_WEIGHT: f32 = 1000.0;
    const RESOLUTION_MATCHING_WEIGHT: f32 = 1000.0;

    let Some(resolution) = search_result.resolution.clone() else {
        // The image has no resolution specified
        return MIN;
    };
    let (width, height): (u32, u32) = {
        let Ok(tmp): Result<Vec<_>, _> = resolution
            .split(|c: char| !c.is_ascii_digit())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::parse)
            .try_collect()
        else {
            debug!("weird resolution format found: {resolution}");
            return MIN;
        };
        if tmp.len() != 2 {
            debug!(
                "weird resolution format found: {resolution} (parsed = {:?})",
                &tmp
            );
            return MIN;
        }
        (tmp[0], tmp[1])
    };
    if width < IMAGE_SIZE || height < IMAGE_SIZE {
        // The image is smaller than required
        return MIN;
    }

    // ***********************
    // * 1. SQUARINESS SCORE *
    // ***********************
    let max_len = width.max(height) as f32;
    let min_len = width.min(height) as f32;
    // 0..1
    let aspect_ratio = min_len / max_len;
    let squariness_score = SQUARINESS_WEIGHT * aspect_ratio;

    // *******************
    // * 2. FORMAT SCORE *
    // *******************
    // png, svg, jpeg etc or empty string if we don't know
    let file_ext = search_result
        .img_format
        .clone()
        .or_else(|| search_result.img_src.split('.').last().map(str::to_owned))
        .unwrap_or_default();
    let format_score = if file_ext == "png" || file_ext == "svg" {
        FORMAT_WEIGHT
    } else {
        0.0
    };
    // **********************************
    // * 3. RESOLUTION MATCHING SCORE   *
    // **********************************
    // 0..1
    let resolution_match_ratio = (IMAGE_SIZE + IMAGE_SIZE) as f32 / (width + height) as f32;
    let resolution_matching_score = resolution_match_ratio * RESOLUTION_MATCHING_WEIGHT;
    // ****************************
    // * 4. SEARCH ENGINE SCORE   *
    // ****************************
    let search_engine_score = search_result.score * SEARCH_ENGINE_RANK_WEIGHT;

    
    squariness_score + format_score + search_engine_score + resolution_matching_score
}

async fn find_icon(application_name: &str) -> anyhow::Result<DynamicImage> {
    // 1. send request to searxng api eg https://search.eldolfin.top/search?q=!images+satisfactory+logo+square&category_images=
    // 2. rank result by (from the most important to the less important)
    //   - resolution
    //     - it should be close enough to the expected icon size (but larger)
    //     - it should be as square as possible
    //   - format
    //     - it should be png or svg
    //   - search engine rank

    let search_query = format!("{application_name} logo");
    let response = searxng_api::query_image(search_query).await?;
    let best_icon = response
        .results
        .into_iter()
        .map(IconMetadata::from)
        .max_by_key(|icon| ComparableFloats::from(icon.score))
        .expect("Searxng to return some results");
    debug!("Found best icon with score {}", best_icon.score);
    // debug!("Icon: {:#?}", best_icon.search_result);
    let image = download_image(&best_icon.search_result.img_src).await?;
    Ok(image)
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use crate::cache::icon::find_icon;
    use crate::test::logfxt;

    #[rstest]
    #[case("Discord")]
    #[case("firefox")]
    #[case("firefox")]
    #[tokio::test]
    async fn test_find_icon(logfxt: (), #[case] appname: String) {
        find_icon(&appname).await;
    }
}
