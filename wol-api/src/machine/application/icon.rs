use image::DynamicImage;

pub fn find_icon(application_name: &str) -> DynamicImage {
    // 1. send request to searxng api eg https://search.eldolfin.top/search?q=!images+satisfactory+logo+square&category_images=
    // 2. rank result by (from the most important to the less important)
    //   - resolution
    //     - it should be close enough to the expected icon size (but larger)
    //     - it should be as square as possible
    //   - format
    //     - it should be png or svg
    //   - search engine rank
    todo!()
}

struct IconMetadata {}
