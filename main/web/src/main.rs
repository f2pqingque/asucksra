use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use asucksra_crawler::get_manhwa_chapter_img_urls;
use std::net::SocketAddr;

async fn asu(
    Path((name, chapter)): Path<(String, u16)>,
) -> impl IntoResponse {
    let images = match get_manhwa_chapter_img_urls(&name, chapter).await {
        Ok(Some(v)) => {
            if v.is_empty() {
                vec![String::from("couldn't find anything :(")]
            } else {
                v
            }
        }
        _ => vec![String::from("couldn't find anything :(")],
    };

    let mut images_for_html = Vec::new();

    for img in images {
        images_for_html.push(format!(r#"<img src="{}">"#, img));
    }

    let images_html = images_for_html.join("\n");

    let html = format!(
        r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>hi</title>
                <style>
                    body {{
                        background-color: #000;
                        margin: 0;
                        padding: 0;
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                    }}

                    .image-container {{
                        display: flex;
                        flex-direction: column;
                    }}

                    .image-container img {{
                        display: block;
                        margin: 0;
                    }}
                </style>
            </head>
            <body>
                <div class="image-container">
                    {}
                </div>
            </body>
            </html>
        "#,
        images_html
    );

    Html(html)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/:name/:chapter", get(asu));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    axum_server::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
