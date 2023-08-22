use serde::Deserialize;

#[derive(Deserialize)]
pub struct StorefrontAttributes {
    #[serde(rename = "supportedLanguageTags")]
    pub supported_language_tags: Vec<String>,
    #[serde(rename = "explicitContentPolicy")]
    pub explicit_content_policy: String,
    pub name: String,
    #[serde(rename = "defaultLanguageTag")]
    pub default_language_tag: String,
}

#[derive(Deserialize)]
pub struct Storefront {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String, // r#type if you like
    pub href: String,
    pub attributes: StorefrontAttributes,
}

#[derive(Deserialize)]
pub struct Storefronts {
    pub data: Vec<Storefront>,
}

// 因为好多字段 我就要两个
#[derive(Deserialize)]
pub struct SimpleCatlogAttributes {
    pub name: String,
    #[serde(rename = "artistName")]
    pub artist_name: String,
}

#[derive(Deserialize)]
pub struct SimpleLyricsAttribute {
    pub ttml: String,
}

#[derive(Deserialize)]
pub struct SimpleLyricsData {
    pub attributes: SimpleLyricsAttribute,
}

#[derive(Deserialize)]
pub struct SimpleLyrics {
    pub data: Vec<SimpleLyricsData>,
}

#[derive(Deserialize)]
pub struct SimpleRelationships {
    pub lyrics: SimpleLyrics,
    #[serde(rename = "syllable-lyrics")]
    pub syllable_lyrics: SimpleLyrics,
}

impl SimpleRelationships {
    pub fn get_lyrics(&self, syllable: bool) -> &SimpleLyrics {
        if syllable {
            &self.syllable_lyrics
        } else {
            &self.lyrics
        }
    }
}

#[derive(Deserialize)]
pub struct SimpleAlbumTrack {
    pub attributes: SimpleCatlogAttributes,
    pub relationships: SimpleRelationships,
}

#[derive(Deserialize)]
pub struct SimpleAlbumTracks {
    pub data: Vec<SimpleAlbumTrack>,
}

// 主要取歌词相关的
#[derive(Deserialize)]
pub struct SimpleAlbumRelationships {
    pub tracks: SimpleAlbumTracks,
}

#[derive(Deserialize)]
pub struct AlbumCatlogData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String, // r#type if you like
    pub href: String,
    pub attributes: SimpleCatlogAttributes,
    pub relationships: SimpleAlbumRelationships,
}

#[derive(Deserialize)]
pub struct AlbumCatlogs {
    pub data: Vec<AlbumCatlogData>,
}

#[derive(Deserialize)]
pub struct SongCatlogData {
    pub id: String,
    #[serde(rename = "type")]
    pub data_type: String, // r#type if you like
    pub href: String,
    pub attributes: SimpleCatlogAttributes,
    pub relationships: SimpleRelationships,
}

#[derive(Deserialize)]
pub struct SongCatlogs {
    pub data: Vec<SongCatlogData>,
}
