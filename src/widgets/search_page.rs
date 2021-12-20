use crate::glib_utils::{RustedListBox, RustedListStore};
use crate::invidious::core::{Channel, TrendingVideo};
use crate::widgets::{RemoteImageExt, VideoRow};
use crate::Client;
use anyhow::anyhow;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use once_cell::sync::OnceCell;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/search_page.ui")]
    pub struct SearchPage {
        #[template_child]
        pub video_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        pub video_list_model: RustedListStore<TrendingVideo>,
        pub client: OnceCell<Client>,
    }

    impl Default for SearchPage {
        fn default() -> Self {
            Self {
                video_list: TemplateChild::default(),
                video_list_model: RustedListStore::new(),
                search_entry: TemplateChild::default(),
                client: OnceCell::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchPage {
        const NAME: &'static str = "SearchPage";
        type Type = super::SearchPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl WidgetImpl for SearchPage {}
    impl ObjectImpl for SearchPage {}
    impl BoxImpl for SearchPage {}
}

glib::wrapper! {
    pub struct SearchPage(ObjectSubclass<imp::SearchPage>)
        @extends gtk::Widget, gtk::Box;
}

impl SearchPage {
    pub fn new(client: Client) -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create SearchPage");

        let self_ = obj.impl_();
        self_.client.set(client).unwrap();
        obj.prepare_widgets();
        obj
    }
    pub fn set_client(&self, client: Client) {
        let self_ = self.impl_();
        self_.client.set(client).unwrap();
    }
    fn prepare_widgets(&self) {
        let self_ = self.impl_();
        self_
            .video_list
            .bind_rusted_model(&self_.video_list_model, |v| {
                VideoRow::new(v.clone()).upcast()
            });
        self_.video_list.connect_row_activated(|_, row| {
            let row: VideoRow = row.clone().downcast().unwrap();
            row.activate_action("win.view-video", Some(&row.video().video_id.to_variant()))
                .unwrap();
        });
        let client = self_.client.get().unwrap();
        self_
            .search_entry
            .connect_search_changed(clone!(@strong client,
                @strong self_.video_list_model as video_list_model => move |entry| {
                let text = entry.text();
                let client = client.clone();
                let video_list_model = video_list_model.clone();
                glib::MainContext::default().spawn_local(async move {
                    let res = client.search(&text).await.unwrap();
                    video_list_model.extend(res.into_iter());
                });
            }));
    }
}
