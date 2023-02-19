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
pub fn to_http_response(path: &String) -> HttpResponse {
    let mut try_files: [String; 3] = [
        path.to_owned(),                 // Attempt the given path
        path.to_owned() + ".html",       // Maybe we forgot the .html suffix?
        path.to_owned() + "/index.html", // Maybe we requested a directory, expecting the index.html file?
    ];

    if path.is_empty() {
        try_files[0] = "index.html".to_owned();
    }

    for filename in try_files {
        match HtmlAssets::get(&filename) {
            Some(file) => {
                // Guess the Content-Type using the filename
                // n.b. fallback to "application/octet-stream" if unknown
                let mime_type = mime_guess::from_path(filename).first_or_octet_stream();

                return HttpResponse::Ok()
                    .content_type(mime_type.as_ref())
                    .body(file.data.into_owned());
            }

            // File not found - try the next possible filename
            None => continue,
        }
    }

    HttpResponse::NotFound().body("404 Not Found")
}
