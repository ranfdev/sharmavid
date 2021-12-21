use crate::invidious::core::*;
use once_cell::sync::Lazy;
use surf::Url;

#[derive(Clone, Debug)]
pub struct Client {
    http: surf::Client,
    base: String,
}

static INSTANCE: Lazy<Client> = Lazy::new(|| Client::default());

impl Default for Client {
    fn default() -> Self {
        Client::new("https://inv.riverside.rocks".to_string()).unwrap()
    }
}

impl Client {
    pub fn new(base: String) -> anyhow::Result<Self> {
        Ok(Self {
            http: surf::Config::new()
                .set_base_url(Url::parse(&format!("{}/api/v1/", &base))?)
                .try_into()?,
            base,
        })
    }
    pub fn global() -> &'static Self {
        &*INSTANCE
    }
    pub async fn popular(&self) -> surf::Result<Vec<TrendingVideo>> {
        self.http.get("popular").recv_json().await
    }
    pub async fn comments(&self, video_id: &str) -> surf::Result<Comments> {
        self.http
            .get(&format!("comments/{}", video_id))
            .recv_json()
            .await
    }
    pub async fn channel(&self, channel_id: &str) -> surf::Result<Channel> {
        self.http
            .get(&format!("channels/{}", channel_id))
            .recv_json()
            .await
    }
    pub async fn video(&self, id: &str) -> surf::Result<FullVideo> {
        self.http.get(&format!("videos/{}", id)).recv_json().await
    }
    pub async fn search(&self, query: &str) -> surf::Result<Vec<TrendingVideo>> {
        self.http
            .get(dbg!(&format!("search/q?={}", query)))
            .recv_json()
            .await
    }
    pub fn base(&self) -> String {
        self.base.clone()
    }
}
