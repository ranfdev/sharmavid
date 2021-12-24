use crate::ev_stream;
use crate::glib_utils::{RustedListBox, RustedListStore};
use crate::invidious::core::{SearchParams, TrendingVideo};
use crate::widgets::VideoRow;
use crate::{Client, Paged};

use futures::join;
use futures::prelude::*;
use futures::stream::Stream;
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/search_page.ui")]
    pub struct SearchPage {
        #[template_child]
        pub video_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        pub video_list_model: RustedListStore<TrendingVideo>,
        pub results: RefCell<Paged<Vec<TrendingVideo>>>,
    }

    impl Default for SearchPage {
        fn default() -> Self {
            Self {
                video_list: TemplateChild::default(),
                video_list_model: RustedListStore::new(),
                search_entry: TemplateChild::default(),
                scrolled_window: TemplateChild::default(),
                results: RefCell::new(Box::pin(stream::empty())),
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
    pub fn new() -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create SearchPage");

        obj.prepare_widgets();
        obj
    }
    fn prepare_widgets(&self) {
        let self_ = self.impl_();
        self_
            .video_list
            .bind_rusted_model(&self_.video_list_model, |v| {
                VideoRow::new(v.clone()).upcast()
            });

        let this = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let self_ = this.impl_();

            join!(
                ev_stream!(self_.scrolled_window, edge_reached, |win, edge|)
                    .filter(|(_, edge)| future::ready(*edge == gtk::PositionType::Bottom))
                    .filter_map(|_| async move { self_.results.borrow_mut().next().await })
                    .filter_map(|res| future::ready(res.ok()))
                    .map(|res: Vec<TrendingVideo>| self_.video_list_model.extend(res.into_iter()))
                    .count(),
                ev_stream!(self_.search_entry, search_changed, |entry|)
                    .for_each(|_| this.handle_search_changed()),
                ev_stream!(self_.video_list, row_activated, |list, row|)
                    .for_each(|(_, row)| this.handle_row_activated(row))
            );
        });
    }
    async fn handle_search_changed(&self) {
        let self_ = self.impl_();

        let mut params = SearchParams::default();
        params.query = self_.search_entry.text().to_string();

        *self_.results.borrow_mut() = Client::global().search(params);
        self_.video_list_model.clear();
        let vids = self_.results.borrow_mut().next().await.unwrap();
        self_.video_list_model.extend(vids.unwrap().into_iter());
    }
    async fn handle_row_activated(&self, row: gtk::ListBoxRow) {
        let row: VideoRow = row.clone().downcast().unwrap();
        row.activate_action("win.view-video", Some(&row.video().video_id.to_variant()))
            .unwrap()
    }
}
