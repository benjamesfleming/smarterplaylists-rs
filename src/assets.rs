use actix_web::HttpResponse;
use mime_guess;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/html"]
struct HtmlAssets;

/// Returns an HttpResponse containing eaither the requested asset, or 404 Not Found
///
/// # Arguments
///
/// * `path` - The asset path relative to ./src/html
///
/// # Examples
///
/// ```
/// assets::to_http_response("index.html")
/// ````
pub fn to_http_response(path: &str) -> HttpResponse {
    match HtmlAssets::get(path) {
        Some(file) => {
            // Guess the Content-Type using the filename
            // n.b. fallback to "application/octet-stream" if unknown
            let mime_type = mime_guess::from_path(path).first_or_octet_stream();

            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(file.data.into_owned())
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}
