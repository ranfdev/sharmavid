use crate::config::{APP_ID, PROFILE};
use crate::ev_stream;
use crate::glib_utils::{RustedListBox, RustedListStore};
use crate::invidious::core::TrendingVideo;
use crate::widgets::{ChannelPage, MiniPlayer, SearchPage, VideoPage, VideoRow};
use crate::Client;
use adw::subclass::prelude::*;
use futures::join;
use futures::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libadwaita as adw;
use std::cell::Cell;
use std::rc::Rc;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/window.ui")]
    pub struct SharMaVidWindow {
        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub video_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub over_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub video_over_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub mini_player: TemplateChild<MiniPlayer>,
        #[template_child]
        pub video_page: TemplateChild<VideoPage>,
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,
        pub video_list_model: RustedListStore<TrendingVideo>,
        pub settings: gio::Settings,
    }

    impl Default for SharMaVidWindow {
        fn default() -> Self {
            Self {
                headerbar: TemplateChild::default(),
                video_list: TemplateChild::default(),
                stack: TemplateChild::default(),
                over_stack: TemplateChild::default(),
                video_over_stack: TemplateChild::default(),
                video_list_model: RustedListStore::new(),
                mini_player: TemplateChild::default(),
                overlay: TemplateChild::default(),
                video_page: TemplateChild::default(),
                settings: gio::Settings::new(APP_ID),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SharMaVidWindow {
        const NAME: &'static str = "SharMaVidWindow";
        type Type = super::SharMaVidWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SharMaVidWindow {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Load latest window state
            obj.load_window_size();
        }
    }

    impl WidgetImpl for SharMaVidWindow {}
    impl WindowImpl for SharMaVidWindow {
        // Save window state on delete event
        fn close_request(&self, window: &Self::Type) -> gtk::Inhibit {
            if let Err(err) = window.save_window_size() {
                log::warn!("Failed to save window state, {}", &err);
            }

            // Pass close request on to the parent
            self.parent_close_request(window)
        }
    }

    impl ApplicationWindowImpl for SharMaVidWindow {}
    impl AdwApplicationWindowImpl for SharMaVidWindow {}
}

glib::wrapper! {
    pub struct SharMaVidWindow(ObjectSubclass<imp::SharMaVidWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl SharMaVidWindow {
    pub fn new(app: &adw::Application) -> Self {
        let obj: Self =
            glib::Object::new(&[("application", app)]).expect("Failed to create SharMaVidWindow");
        obj.setup_actions();
        obj.setup_widgets();
        obj
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let self_ = self.impl_();

        let (width, height) = self.default_size();

        self_.settings.set_int("window-width", width)?;
        self_.settings.set_int("window-height", height)?;

        self_
            .settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }
    pub fn setup_actions(&self) {
        let back = gio::SimpleAction::new("back", None);
        self.add_action(&back);
        let show_video = gio::SimpleAction::new("view-video", Some(glib::VariantTy::STRING));
        self.add_action(&show_video);
        let minimize_video = gio::SimpleAction::new("minimize-video", None);
        self.add_action(&minimize_video);
        let unminimize_video = gio::SimpleAction::new("unminimize-video", None);
        self.add_action(&unminimize_video);
        let show_channel = gio::SimpleAction::new("view-channel", Some(glib::VariantTy::STRING));
        self.add_action(&show_channel);
        let show_search = gio::SimpleAction::new("view-search", None);
        self.add_action(&show_search);

        let this = self.clone();
        glib::MainContext::default().spawn_local(async move {
            join!(
                ev_stream!(show_search, activate, |_target, data| data.cloned())
                    .for_each(|_| this.show_search()),
                ev_stream!(show_channel, activate, |_target, channel_id| channel_id
                    .unwrap()
                    .get()
                    .unwrap())
                .for_each(|id| this.show_channel(id)),
                ev_stream!(show_video, activate, |_target, video_id| video_id
                    .unwrap()
                    .get()
                    .unwrap())
                .for_each(|id| this.show_video(id)),
                ev_stream!(minimize_video, activate, |_target, data| data.cloned())
                    .for_each(|_| future::ready(this.minimize_video())),
                ev_stream!(unminimize_video, activate, |_target, data| data.cloned())
                    .for_each(|_| future::ready(this.unminimize_video())),
                ev_stream!(back, activate, |_target, _data| ()).for_each(|_| this.back())
            );
        });
    }
    pub async fn show_channel(&self, channel_id: String) {
        let self_ = self.impl_();
        self.minimize_video();
        let page = ChannelPage::new();
        self_.over_stack.add_child(&page);
        self_.over_stack.set_visible_child(&page);
        let channel = Client::global().channel(&channel_id).await.unwrap();
        page.set_channel(channel);
    }
    pub async fn show_video(&self, video_id: String) {
        let self_ = self.impl_();
        self.unminimize_video();
        let video = Client::global().video(&video_id).await.unwrap();
        let video_page = self_.video_page.clone();
        let mp = self_.mini_player.clone();
        glib::source::idle_add_local(move || {
            video_page.set_video(video.clone());
            mp.set_video(video.clone());
            Continue(false)
        });

        // TODO: Put this in a method .as_trending in invidious/core.rs
        /*let trending_video = TrendingVideo {
            title: video.title,
            video_id: video.video_id,
            author: video.author,
            author_url: video.author_url,
            author_id: video.author_id,
            view_count: video.view_count,
            video_thumbnails: video.video_thumbnails,
            length_seconds: video.length_seconds,
            published: video.published,
            published_text: video.published_text,
            description: Some(video.description),
            description_html: Some(video.description_html),
        };*/
    }
    pub fn unminimize_video(&self) {
        let self_ = self.impl_();
        self_.video_over_stack.set_visible_child(&*self_.video_page);
    }
    pub fn minimize_video(&self) {
        let self_ = self.impl_();
        self_.video_over_stack.set_visible_child(&*self_.overlay);
    }
    pub async fn show_search(&self) {
        let self_ = self.impl_();
        let search_page = SearchPage::new();
        self_.over_stack.add_child(&search_page);
        self_.over_stack.set_visible_child(&search_page);
    }
    pub fn setup_widgets(&self) {
        let self_ = self.impl_();
        self_
            .video_list
            .bind_rusted_model(&self_.video_list_model, move |v| {
                VideoRow::new(v.clone()).upcast()
            });
        self_.video_list.connect_row_activated(|_, row| {
            let row: VideoRow = row.clone().downcast().unwrap();
            row.activate_action("win.view-video", Some(&row.video().video_id.to_variant()))
                .unwrap();
        });
    }

    pub async fn back(&self) {
        let self_ = self.impl_();
        let stack = self_.over_stack.clone();

        let pages = &stack
            .pages()
            .snapshot()
            .into_iter()
            .flat_map(|obj| obj.clone().downcast::<gtk::StackPage>().ok())
            .map(|obj| obj.child())
            .collect::<Vec<gtk::Widget>>()[..];
        match pages {
            [.., prev_page, curr_page] => {
                stack.set_visible_child(prev_page);
                let curr_page = curr_page.clone();
                let signal_rc = Rc::new(Cell::new(None));
                let cloned_signal_rc = signal_rc.clone();
                signal_rc.set(Some(stack.connect_transition_running_notify({
                    let stack = stack.clone();
                    move |_| {
                        if !stack.is_transition_running() {
                            stack.remove(&curr_page);
                            stack.disconnect(cloned_signal_rc.take().unwrap());
                        }
                    }
                })));
            }
            _ => log::warn!("No pages to go back to"),
        }
    }
    fn load_window_size(&self) {
        let self_ = self.impl_();

        let width = self_.settings.int("window-width");
        let height = self_.settings.int("window-height");
        let is_maximized = self_.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
    pub fn load_popular(&self) {
        let self_ = self.impl_();
        let video_list_model = self_.video_list_model.clone();
        glib::MainContext::default().spawn_local(async move {
            video_list_model.extend(
                Client::global()
                    .popular()
                    .await
                    .map_or(vec![].into_iter(), |v| v.into_iter()),
            );
        });
    }
}
