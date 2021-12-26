use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrendingVideo {
    pub title: String,
    pub video_id: String,
    pub video_thumbnails: Vec<VideoThumbnail>,
    pub length_seconds: i32,
    pub view_count: i64,
    pub author: String,
    pub author_id: String,
    pub author_url: String,
    pub published: i64,
    pub published_text: String,
    pub description: Option<String>,
    pub description_html: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveFormat {
    pub index: String,
    pub bitrate: String,
    pub init: String,
    pub url: String,
    pub itag: String,
    //type: String,
    pub clen: String,
    pub lmt: String,
    pub container: Option<String>,
    pub encoding: Option<String>,
    pub quality_label: Option<String>,
    pub resolution: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FullVideo {
    pub title: String,
    pub video_id: String,
    pub video_thumbnails: Vec<VideoThumbnail>,
    pub length_seconds: i32,
    pub view_count: i64,
    pub author: String,
    pub author_id: String,
    pub author_url: String,
    pub author_thumbnails: Vec<SizedImage>,
    pub published: i64,
    pub published_text: String,
    pub description: String,
    pub description_html: String,
    pub hls_url: Option<String>,
    pub adaptive_formats: Vec<AdaptiveFormat>,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct VideoThumbnail {
    pub quality: String,
    pub url: String,
    pub width: i32,
    pub height: i32,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SizedImage {
    pub url: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Replies {
    pub reply_count: i32,
    pub continuation: String,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub author: String,
    pub author_thumbnails: Vec<SizedImage>,
    pub author_id: String,
    pub author_url: String,
    pub is_edited: bool,
    pub content: String,
    pub content_html: String,
    pub published: i64,
    pub published_text: String,
    pub like_count: i32,
    pub comment_id: String,
    pub author_is_channel_owner: bool,
    pub replies: Option<Replies>,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Comments {
    pub comment_count: Option<i32>,
    pub video_id: String,
    pub comments: Vec<Comment>,
    pub continuation: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BasicAuthor {
    pub author: String,
    pub author_id: String,
    pub author_url: String,
    pub author_thumbnails: Vec<SizedImage>,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub author: String,
    pub author_id: String,
    pub author_url: String,
    pub author_banners: Vec<SizedImage>,
    pub author_thumbnails: Vec<SizedImage>,
    pub sub_count: i32,
    pub total_views: i64,
    pub joined: i64,
    pub is_family_friendly: bool,
    pub description: String,
    pub description_html: String,
    pub allowed_regions: Vec<String>,
    pub latest_videos: Vec<TrendingVideo>,
    pub related_channels: Vec<BasicAuthor>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Relevance,
    Rating,
    UploadDate,
    ViewCount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DateQuery {
    Hour,
    Today,
    Week,
    Month,
    Year,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Duration {
    Short,
    Long,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    Video,
    Playlist,
    Channel,
    All,
}

#[derive(Debug, Serialize, Deserialize, Clone, Builder, Default)]
pub struct SearchParams {
    pub query: String,
    #[builder(default)]
    pub page: Option<i32>,
    #[builder(default)]
    pub sort_by: Option<SortBy>,
    #[builder(default)]
    pub date: Option<DateQuery>,
    #[builder(default)]
    pub duration: Option<Duration>,
    #[builder(default)]
    #[serde(rename = "type")]
    pub search_type: Option<SearchType>, // features: ...
                                         // region: ...
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CommentsSortBy {
    Top,
    New,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CommentsSource {
    Youtube,
    Reddit,
}
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CommentsParams {
    pub video_id: String,
    pub sort_by: Option<CommentsSortBy>,
    pub source: Option<CommentsSource>,
    pub continuation: Option<String>,
}
