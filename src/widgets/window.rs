use crate::config::{APP_ID, PROFILE};
use crate::glib_utils::RustedListModel;
use crate::invidious::core::TrendingVideo;
use crate::widgets::{ChannelPage, VideoPage, VideoRow};
use crate::Client;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libadwaita as adw;
use log::warn;
use once_cell::sync::OnceCell;
use std::cell::Cell;
use std::rc::Rc;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/window.ui")]
    pub struct SharMaVidWindow {
        #[template_child]
        pub headerbar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub video_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub back_btn: TemplateChild<gtk::Button>,
        pub video_list_model: RustedListModel<TrendingVideo>,
        pub settings: gio::Settings,
        pub client: OnceCell<Client>,
    }

    impl Default for SharMaVidWindow {
        fn default() -> Self {
            Self {
                headerbar: TemplateChild::default(),
                video_list: TemplateChild::default(),
                stack: TemplateChild::default(),
                back_btn: TemplateChild::default(),
                video_list_model: RustedListModel::new(),
                settings: gio::Settings::new(APP_ID),
                client: OnceCell::new(),
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
    pub fn new(app: &adw::Application, client: Client) -> Self {
        let obj: Self =
            glib::Object::new(&[("application", app)]).expect("Failed to create SharMaVidWindow");
        let self_ = obj.impl_();
        self_.client.set(client).unwrap();

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
        back.connect_activate(clone!(@strong self as this => move |_, _| this.back()));
        self.add_action(&back);

        let show_video = gio::SimpleAction::new("view-video", Some(glib::VariantTy::STRING));
        show_video.connect_activate(clone!(@strong self as this => move |_, video_id| this.show_video(video_id.unwrap().get().unwrap())));
        self.add_action(&show_video);

        let show_channel = gio::SimpleAction::new("view-channel", Some(glib::VariantTy::STRING));
        show_channel.connect_activate(clone!(@strong self as this => move |_, channel_id| this.show_channel(channel_id.unwrap().get().unwrap())));
        self.add_action(&show_channel);
    }
    pub fn show_channel(&self, channel_id: String) {
        let cloned_self = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let self_ = cloned_self.impl_();
            let client = self_.client.get().unwrap();
            let page = ChannelPage::new(client.clone());
            self_.stack.add_child(&page);
            self_.stack.set_visible_child(&page);
            let channel = client.channel(&channel_id).await.unwrap();
            page.set_channel(channel);
        });
    }
    pub fn show_video(&self, video_id: String) {
        let cloned_self = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let self_ = cloned_self.impl_();
            let client = self_.client.get().unwrap();
            let page = VideoPage::new(client.clone());
            self_.stack.add_child(&page);
            self_.stack.set_visible_child(&page);
            let video = client.video(&video_id).await.unwrap();
            page.set_video(video);
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
        });
    }
    pub fn setup_widgets(&self) {
        let self_ = self.impl_();
        self_
            .video_list_model
            .bind_to_list_box(&*self_.video_list, move |v| VideoRow::new(v).upcast());
    }

    pub fn back(&self) {
        let self_ = self.impl_();
        let stack = &self_.stack;
        let model = stack.pages();
        let n = model.n_items();
        let pages: [Option<gtk::Widget>; 2] = [
            model.item(n.overflowing_sub(2).0),
            model.item(n.overflowing_sub(1).0),
        ]
        .map(|page| {
            page.map(|p| {
                p.downcast::<gtk::StackPage>()
                    .expect("Not a gtk::StackPage")
                    .child()
            })
        });
        match pages {
            [Some(prev_page), Some(curr_page)] => {
                stack.set_visible_child(&prev_page);
                let curr_page = curr_page.clone();
                let signal_rc = Rc::new(Cell::new(None));
                let cloned_signal_rc = signal_rc.clone();
                signal_rc.set(Some(stack.connect_transition_running_notify({
                    let stack = self_.stack.clone();
                    move |_| {
                        if !stack.is_transition_running() {
                            stack.remove(&curr_page);
                            stack.disconnect(cloned_signal_rc.take().unwrap());
                        }
                    }
                })));
            }
            _ => warn!("No pages to go back to"),
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
        let client = self_.client.get().unwrap().clone();
        glib::MainContext::default().spawn_local(async move {
            video_list_model.extend(
                client
                    .popular()
                    .await
                    .map_or(vec![].into_iter(), |v| v.into_iter()),
            );
        });
    }
}
