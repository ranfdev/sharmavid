use crate::invidious::core::*;
use futures::prelude::*;
use futures::stream::Stream;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::pin::Pin;
use surf::Url;

#[derive(Clone, Debug)]
pub struct Client {
    http: surf::Client,
    base: String,
}

pub type Paged<T> = Pin<Box<dyn Stream<Item = surf::Result<T>>>>;

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
    pub fn comments(&self, video_id: String, params: CommentsParams) -> Paged<Comments> {
        let this = self.clone();
        stream::unfold(
            params.continuation.clone(),
            move |continuation: Option<String>| {
                let this = this.clone();
                let mut params = params.clone();
                params.continuation = continuation;
                let video_id = video_id.clone();
                async move {
                    let url = &format!(
                        "comments/{}?{}",
                        video_id,
                        serde_urlencoded::to_string(&params).unwrap()
                    );
                    log::info!("fetching {}", url);
                    let res = this.http.get(url).recv_json().await;
                    let continuation = res
                        .as_ref()
                        .ok()
                        .map(|comments: &Comments| comments.continuation.clone())
                        .flatten();
                    Some((res, continuation))
                }
            },
        )
        .boxed()
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
    pub fn search(&self, query: SearchParams) -> Paged<Vec<TrendingVideo>> {
        let this = self.clone();
        stream::unfold(query.page.unwrap_or(1), move |state| {
            let this = this.clone();
            let mut params = query.clone();
            params.page = Some(state);
            async move {
                let url = &format!("search/?{}", serde_urlencoded::to_string(&params).unwrap());
                log::info!("fetching {}", url);
                let res: surf::Result<Vec<TrendingVideo>> = this.http.get(url).recv_json().await;
                Some((res, state + 1))
            }
        })
        .boxed()
    }
    pub fn base(&self) -> String {
        self.base.clone()
    }
}
