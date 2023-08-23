mod types;

use crate::types::{AlbumCatlogs, SongCatlogs, Storefronts};
use anyhow::{Context, Result};
use reqwest::{header, Client, Error, Response, Url};
use std::io::Write;
use std::str::FromStr;
use xmlem::display::Config;
use xmlem::Document;

use clap::Parser;

pub fn nice_xml(xml: String) -> String {
    Document::from_str(&xml)
        .expect("Failed to parse xml")
        .to_string_pretty_with_config(&Config {
            is_pretty: true,
            indent: 2,
            end_pad: 1,
            max_line_length: 128,
            entity_mode: Default::default(),
            indent_text_nodes: false,
        })
}

struct Uta {
    client: Client,
    token: String,
    access_token: String,
    store_front: String,
    language: String,
}

#[derive(Parser)]
#[clap(version, name = "uta")]
struct Options {
    /// URL of the song or album
    #[arg(short = 'u', long = "url")]
    url: String,
    /// Need syllable lyrics
    #[arg(short = 's', long = "syllable")]
    syllable: bool,
}

impl Uta {
    async fn handle_raw_url(&self, url: String, syllable: bool) -> Result<()> {
        let parsed = Url::parse(&url).context("Failed to parse url")?;

        let pairs = parsed.query_pairs();

        if pairs.count() == 0 {
            let id = parsed
                .path_segments()
                .context("Failed to get path segments")?
                .last()
                .context("No path segments")?;
            self.save_album_lyrics(id.to_string(), syllable).await?;
            return Ok(());
        }

        let id = parsed.query_pairs().find(|(key, _)| key == "i");

        if let Some(id) = id {
            self.save_song_lyrics(id.1.to_string(), syllable).await?;
        } else {
            let id = parsed
                .path_segments()
                .context("Failed to get path segments")?
                .last()
                .context("No path segments")?;
            self.save_album_lyrics(id.to_string(), syllable).await?;
        }

        Ok(())
    }

    async fn get_response(&self, url: String) -> std::result::Result<Response, Error> {
        self.client
            .get(&url)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", self.access_token),
            )
            .header("media-user-token", self.token.clone())
            .header(
                header::ACCEPT_LANGUAGE,
                format!("{},en;q=0.9", self.language),
            )
            .query(&[
                ("l", self.language.clone()),
                ("include[songs]", "album,lyrics,syllable-lyrics".to_string()),
            ])
            .send()
            .await
    }

    async fn save_album_lyrics(&self, album_id: String, syllable: bool) -> Result<()> {
        println!("Getting album info...");

        let url = format!(
            "https://amp-api.music.apple.com/v1/catalog/{}/albums/{}",
            self.store_front, album_id
        );

        let result = self
            .get_response(url)
            .await
            .context("Failed to send request to Apple Music")?;

        let album_catlogs: AlbumCatlogs = result.json().await.unwrap();

        let catlog_data = album_catlogs.data.get(0).context("No album found")?;

        let attributes = &catlog_data.attributes;

        let tracks = &catlog_data.relationships.tracks.data;

        println!("Saving lyrics...");

        let folder_name = format!("{} - {}", attributes.name, attributes.artist_name);
        std::fs::create_dir(&folder_name).context("Failed to create folder")?;

        for track in tracks {
            let lyrics = track.relationships.get_lyrics(syllable);
            let file_name = format!(
                "{}/{} - {}.ttml",
                folder_name, track.attributes.name, track.attributes.artist_name
            );
            let track_lyric = lyrics.data.get(0);
            if let Some(lyric) = track_lyric {
                let mut file =
                    std::fs::File::create(&file_name).context("Failed to create file")?;
                file.write_all(nice_xml(lyric.attributes.ttml.clone()).as_bytes())
                    .context("Failed to write file")?;
            } else {
                println!(
                    "{} - {} has no lyrics",
                    track.attributes.name, track.attributes.artist_name
                );
            }
        }

        Ok(())
    }

    async fn save_song_lyrics(&self, song_id: String, syllable: bool) -> Result<()> {
        println!("Getting song info...");

        let url = format!(
            "https://amp-api.music.apple.com/v1/catalog/{}/songs/{}",
            self.store_front, song_id
        );
        let result = self
            .get_response(url)
            .await
            .context("Failed to send request to Apple Music")?;

        let song_catlogs: SongCatlogs = result.json().await.context("Failed to parse json")?;

        let catlog_data = song_catlogs.data.get(0).context("No song found")?;

        let relation = &catlog_data.relationships;
        let attributes = &catlog_data.attributes;

        let lyrics = relation.get_lyrics(syllable);

        println!("Saving lyrics...");

        let lyric = lyrics.data.get(0);

        if let Some(lyric) = lyric {
            let file_name = format!("{} - {}.ttml", attributes.name, attributes.artist_name);
            let mut file = std::fs::File::create(file_name).context("Failed to create file")?;
            file.write_all(nice_xml(lyric.attributes.ttml.clone()).as_bytes())
                .context("Failed to write file")?;
        } else {
            println!("This song has no lyrics");
        }

        Ok(())
    }

    async fn new(token: String) -> Result<Self> {
        println!("Initializing...");

        let mut headers = header::HeaderMap::new();

        headers.append(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json;charset=utf-8"),
        );
        headers.append(
            header::CONNECTION,
            header::HeaderValue::from_static("keep-alive"),
        );
        headers.append(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.append(
            header::ORIGIN,
            header::HeaderValue::from_static("https://music.apple.com"),
        );
        headers.append(
            header::REFERER,
            header::HeaderValue::from_static("https://music.apple.com/"),
        );
        headers.append(
            header::ACCEPT_ENCODING,
            header::HeaderValue::from_static("gzip, deflate, br"),
        );
        headers.append(
            header::USER_AGENT,
            header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36")
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build reqwest client")?;

        let main_page = client
            .get("https://music.apple.com/us/browse")
            .send()
            .await
            .context("Failed to send request to Apple Music")?;

        let main_page_code = main_page.text().await?;
        let js_search_re =
            regex::Regex::new(r#"index(.*?)\.js"#).context("Failed to compile regex")?;
        let js_search = js_search_re
            .captures(&main_page_code)
            .context("Failed to find js file")?;
        let js_file = js_search
            .get(0)
            .context("Failed to get js file")?
            .as_str()
            .to_string();

        let js_file_page = client
            .get(&format!("https://music.apple.com/assets/{}", js_file))
            .send()
            .await
            .context("Failed to send request to Apple Music")?;

        let js_file_code = js_file_page.text().await?;
        let jwt_search_re =
            regex::Regex::new(r#""(?P<key>eyJh(.*?))""#).context("Failed to compile regex")?;
        let jwt_search = jwt_search_re
            .captures(&js_file_code)
            .context("Failed to find jwt")?;
        let jwt = jwt_search["key"].to_string();

        let store_front_rsp = client
            .get("https://amp-api.music.apple.com/v1/me/storefront")
            .header(header::AUTHORIZATION, format!("Bearer {}", jwt))
            .header("media-user-token", token.clone())
            .send()
            .await
            .context("Failed to send request to Apple Music")?;

        // yes, data is list, but we only need first element, strange
        let store_front: Storefronts = store_front_rsp
            .json()
            .await
            .context("Failed to parse json")?;
        let store_id = store_front.data[0].id.clone();
        let language = store_front.data[0].attributes.default_language_tag.clone();

        Ok(Uta {
            client,
            token,
            access_token: jwt,
            store_front: store_id,
            language,
        })
    }
}

#[tokio::main]
async fn main() {
    let args: Options = Options::parse();
    let meta = std::env::var("APPLE_META_TOKEN").expect("APPLE_META_TOKEN is not set");
    let uta = Uta::new(meta).await;
    if let Ok(uta) = uta {
        uta.handle_raw_url(args.url, args.syllable).await.unwrap();
    }
}
