use anyhow::anyhow;
use async_trait::async_trait;
use futures::prelude::*;
use gio::prelude::*;
use gtk::{gdk_pixbuf, gio, glib};
use libadwaita as adw;
use log::error;

pub trait RemoteImage: Clone {
    fn set_remote_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf);
}

#[async_trait(?Send)]
pub trait RemoteImageExt: RemoteImage {
    fn set_image_url(&self, url: String);
    async fn set_image_url_future(&self, url: String);
}

#[async_trait(?Send)]
impl<T: 'static + RemoteImage> RemoteImageExt for T {
    fn set_image_url(&self, url: String) {
        let cloned_self: T = self.clone();
        glib::MainContext::default().spawn_local(async move {
            cloned_self.set_image_url_future(url).await;
        });
    }
    async fn set_image_url_future(&self, url: String) {
        match pixbuf_for_img(url).await {
            Ok(pixbuf) => {
                self.set_remote_pixbuf(pixbuf);
            }
            Err(e) => error!("Failed fetching image {}", e),
        }
    }
}

async fn pixbuf_for_img(url: String) -> anyhow::Result<gdk_pixbuf::Pixbuf> {
    let bytes = surf::get(&url)
        .await
        .map_err(|e| anyhow!(e))?
        .body_bytes()
        .await
        .map_err(|e| anyhow!(e))?;

    let mem_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from_owned(bytes));
    let pixbuf = gdk_pixbuf::Pixbuf::from_stream_future(&mem_stream).await?;

    Ok(pixbuf)
}

impl RemoteImage for gtk::Picture {
    fn set_remote_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf) {
        self.set_pixbuf(Some(&pixbuf));
    }
}

impl RemoteImage for adw::Avatar {
    fn set_remote_pixbuf(&self, pixbuf: gdk_pixbuf::Pixbuf) {
        let pict = gtk::Picture::new();
        pict.set_remote_pixbuf(pixbuf);
        self.set_custom_image(pict.paintable().as_ref());
    }
}
