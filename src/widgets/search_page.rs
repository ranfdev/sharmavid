use crate::glib_utils::{RustedListBox, RustedListStore};
use crate::invidious::core::{SearchParams, SortBy, TrendingVideo};
use crate::widgets::VideoRow;
use crate::{ctx, Client};

use ev_stream_gtk_rs::ev_stream;
use futures::future::RemoteHandle;
use futures::join;
use futures::prelude::*;
use futures::task::LocalSpawnExt;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::unsync::OnceCell;

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
        pub async_handle: OnceCell<Option<RemoteHandle<()>>>,
    }

    impl Default for SearchPage {
        fn default() -> Self {
            Self {
                video_list: TemplateChild::default(),
                video_list_model: RustedListStore::new(),
                search_entry: TemplateChild::default(),
                scrolled_window: TemplateChild::default(),
                async_handle: OnceCell::new(),
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
        let imp = self.imp();

        imp.video_list
            .bind_rusted_model(&imp.video_list_model, |v| VideoRow::new(v.clone()).upcast());

        let search_changed_evs = ev_stream!(imp.search_entry, search_changed, |entry|);
        let row_activated_evs = ev_stream!(imp.video_list, row_activated, |_list, row| row.clone());

        let this = self.downgrade();
        let handle = ctx()
            .spawn_local_with_handle(async move {
                join!(
                    search_changed_evs.fold(None::<RemoteHandle<()>>, |_, _| {
                        Self::handle_search_changed(&this)
                    }),
                    row_activated_evs.for_each(Self::handle_row_activated),
                );
            })
            .ok();
        imp.async_handle.set(handle).unwrap();
        let search_entry = imp.search_entry.clone();
        glib::source::idle_add_local(move || {
            search_entry.grab_focus();
            Continue(false)
        });
    }
    async fn handle_search_changed(this: &glib::WeakRef<Self>) -> Option<RemoteHandle<()>> {
        let this = this.upgrade().unwrap();
        let imp = this.imp();

        imp.video_list_model.clear();

        let mut params = SearchParams::default();
        params.query = imp.search_entry.text().to_string();
        params.sort_by = Some(SortBy::Relevance);

        let event_stream = ev_stream!(imp.scrolled_window, edge_reached, |win, edge|);

        let video_list_model = imp.video_list_model.clone();
        let event_stream = stream::once(async { () }) // Do one initial fetch
            .chain(
                event_stream
                    .filter(|(_, edge)| future::ready(*edge == gtk::PositionType::Bottom))
                    .map(|_| ()),
            )
            .zip(Client::global().search(params))
            .filter_map(|(_, res)| future::ready(res.ok()))
            .for_each(move |res: Vec<TrendingVideo>| {
                future::ready(video_list_model.extend(res.into_iter()))
            });

        ctx().spawn_local_with_handle(event_stream).ok()
    }
    async fn handle_row_activated(row: gtk::ListBoxRow) {
        let row: VideoRow = row.clone().downcast().unwrap();
        row.activate_action("win.view-video", Some(&row.video().video_id.to_variant()))
            .unwrap()
    }
}
