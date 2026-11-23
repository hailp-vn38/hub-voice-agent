use crate::{FetchedImage, ImageFetcher};
use anyhow::{Context, bail};
use std::{
    io::Cursor,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use url::Url;

const MAX_BYTES: usize = 10 * 1024 * 1024;
const MAX_REDIRECTS: usize = 5;

/// Downloads public images with bounded network and decoding resources.
#[derive(Default)]
pub struct HttpImageFetcher;

impl ImageFetcher for HttpImageFetcher {
    fn fetch(
        &self,
        url: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<FetchedImage>> + Send + '_>,
    > {
        Box::pin(async move {
            tokio::time::timeout(Duration::from_secs(20), download(url))
                .await
                .context("image download timed out")?
        })
    }
}

async fn download(url: String) -> anyhow::Result<FetchedImage> {
    download_with(url, |url, addresses| {
        Box::pin(async move {
            let host = url
                .host_str()
                .context("missing image host")?
                .trim_matches(['[', ']']);
            let client = reqwest::Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .resolve_to_addrs(host, &addresses)
                .build()?;
            Ok(client.get(url).send().await?)
        })
    })
    .await
}

async fn download_with<F>(url: String, request: F) -> anyhow::Result<FetchedImage>
where
    F: Fn(
        Url,
        Vec<SocketAddr>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<reqwest::Response>> + Send>,
    >,
{
    let mut url = Url::parse(&url)?;
    for _ in 0..=MAX_REDIRECTS {
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
        {
            bail!("invalid image URL");
        }
        let host = url
            .host_str()
            .context("missing image host")?
            .trim_matches(['[', ']']);
        let port = url.port_or_known_default().context("missing port")?;
        let addresses: Vec<SocketAddr> = tokio::net::lookup_host((host, port)).await?.collect();
        if addresses.is_empty() || addresses.iter().any(|address| !public(address.ip())) {
            bail!("image host resolves to a non-public address");
        }
        let mut response = request(url.clone(), addresses).await?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .context("missing redirect location")?
                .to_str()?;
            url = url.join(location)?;
            continue;
        }
        response = response.error_for_status()?;
        if response
            .content_length()
            .is_some_and(|n| n > MAX_BYTES as u64)
        {
            bail!("image too large");
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > MAX_BYTES {
                bail!("image too large");
            }
            bytes.extend_from_slice(&chunk);
        }
        return tokio::task::spawn_blocking(move || decode(bytes)).await?;
    }
    bail!("too many image redirects")
}

fn decode(bytes: Vec<u8>) -> anyhow::Result<FetchedImage> {
    let format = image::guess_format(&bytes)?;
    if !matches!(
        format,
        image::ImageFormat::Jpeg | image::ImageFormat::Png | image::ImageFormat::WebP
    ) {
        bail!("unsupported image format");
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode()?;
    fn png(image: &image::DynamicImage) -> anyhow::Result<Vec<u8>> {
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, image::ImageFormat::Png)?;
        Ok(out.into_inner())
    }
    Ok(FetchedImage {
        image: png(&image.thumbnail(1920, 1080))?,
        thumbnail: png(&image.thumbnail(320, 180))?,
    })
}

fn public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            let [a, b, c, _] = v.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192 && (b == 168 || (b == 0 && (c == 0 || c == 2))))
                || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(v) => {
            if let Some(v4) = v.to_ipv4_mapped() {
                return public(IpAddr::V4(v4));
            }
            let s = v.segments();
            // Only globally routed unicast. Exclude documentation addresses and the
            // 6to4 (2002::/16) and Teredo (2001::/32) tunnels, which embed IPv4.
            (s[0] & 0xe000) == 0x2000
                && s[0] != 0x2002
                && !(s[0] == 0x2001 && (s[1] == 0x0db8 || s[1] == 0))
        }
    }
}

#[cfg(test)]
#[path = "image_fetch_tests.rs"]
mod tests;
