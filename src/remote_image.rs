use anyhow::anyhow;
use async_trait::async_trait;
use gtk::{gdk_pixbuf, gio, glib};
use libadwaita as adw;
use log::error;

#[async_trait(?Send)]
pub trait RemoteImage {
    async fn set_image_url(&self, _url: String) {}
}

async fn pixbuf_for_img(url: String) -> anyhow::Result<gdk_pixbuf::Pixbuf> {
    let bytes = surf::get(&url)
        .await
        .map_err(|e| anyhow!(e))?
        .body_bytes()
        .await
        .map_err(|e| anyhow!(e))?;

    let mem_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from_owned(bytes));
    let pixbuf = gdk_pixbuf::Pixbuf::from_stream(&mem_stream, Some(&gio::Cancellable::new()))?;
    Ok(pixbuf)
}
#[async_trait(?Send)]
impl RemoteImage for gtk::Picture {
    async fn set_image_url(&self, url: String) {
        let pict = self.clone();
        match pixbuf_for_img(url).await {
            Ok(pixbuf) => {
                pict.set_pixbuf(Some(&pixbuf));
            }
            Err(e) => error!("Failed fetching image {}", e),
        }
    }
}
#[async_trait(?Send)]
impl RemoteImage for adw::Avatar {
    async fn set_image_url(&self, url: String) {
        let avatar = self.clone();
        let pict = gtk::Picture::new();
        pict.set_image_url(url).await;
        avatar.set_custom_image(pict.paintable().as_ref());
    }
}
