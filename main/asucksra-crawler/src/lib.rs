use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;
use std::io::{self, ErrorKind};
use std::sync::LazyLock;

static REQ_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

static HREF_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[href]").expect("Failed initializing HREF_SELECTOR"));

static IMG_JSON_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\\\"url\\\":\\\"(https?://[^"]+?)\\\""#)
        .expect("Failed initializing IMG_JSON_REGEX")
});

static STUPID_FUCKING_WEBSITE: LazyLock<String> = LazyLock::new(|| {
    let hex_bytes: Vec<u8> = vec![
        0x68, 0x74, 0x74, 0x70, 0x73, 0x3a, 0x2f, 0x2f, 0x61, 0x73, 0x75, 0x72, 0x61, 0x63, 0x6f,
        0x6d, 0x69, 0x63, 0x2e, 0x6e, 0x65, 0x74, 0x2f,
    ];
    String::from_utf8(hex_bytes).expect("This should never happen")
});

pub async fn get_html_text(url: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let response = REQ_CLIENT.get(url).send().await?;

    if response.status().is_success() {
        response.text().await.map_err(Into::into)
    } else {
        Err(Box::new(io::Error::new(
            ErrorKind::Other,
            format!(
                "Failed to fetch URL: {}. Status: {}",
                url,
                response.status()
            ),
        )))
    }
}

pub async fn search_manhwas(name: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!("{}series?page=1&name={}", *STUPID_FUCKING_WEBSITE, name);
    get_html_text(&url).await
}

pub async fn get_manhwa_url(name: &str) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let html = search_manhwas(name).await?;
    let normalized_name = name.replace(" ", "-").to_lowercase();
    let document = Html::parse_document(&html);

    for element in document.select(&HREF_SELECTOR) {
        if let Some(href) = element.value().attr("href") {
            if href.contains(&normalized_name) {
                return Ok(Some(format!("{}/{}", *STUPID_FUCKING_WEBSITE, href)));
            }
        }
    }

    Err(Box::new(io::Error::new(
        ErrorKind::NotFound,
        format!("Manhwa not found for name: {}", name),
    )))
}

pub async fn get_manhwa_chapter_img_urls(
    name: &str,
    chapter: u16,
) -> Result<Option<Vec<String>>, Box<dyn Error + Send + Sync>> {
    if let Some(manhwa_url) = get_manhwa_url(name).await? {
        let chapter_url = format!("{}/chapter/{}", manhwa_url, chapter);
        let html = get_html_text(&chapter_url).await?;

        let mut img_sources = Vec::new();
        let mut first_image_found = false;

        for cap in IMG_JSON_REGEX.captures_iter(&html) {
            if let Some(matched_url) = cap.get(1) {
                let img_url = matched_url.as_str().to_string();
                if img_url.contains("01-optimized") {
                    if first_image_found {
                        break;
                    } else {
                        first_image_found = true;
                    }
                }
                img_sources.push(img_url);
            }
        }

        Ok(Some(img_sources))
    } else {
        Err(Box::new(io::Error::new(
            ErrorKind::NotFound,
            format!("Manhwa URL not found for name: {}", name),
        )))
    }
}
