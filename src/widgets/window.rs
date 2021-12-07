use crate::config::{APP_ID, PROFILE};
use crate::glib_utils::RustedListModel;
use crate::invidious::core::TrendingVideo;
use crate::widgets::{ChannelPage, VideoPage, VideoRow};
use crate::Client;
use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use libadwaita as adw;
use once_cell::sync::OnceCell;

pub enum Action {
    ShowVideo(TrendingVideo),
    ShowChannelByID(String),
}

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
        pub action_pusher: OnceCell<glib::Sender<Action>>,
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
                action_pusher: OnceCell::new(),
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
        let self_ = imp::SharMaVidWindow::from_instance(&obj);
        self_.client.set(client).unwrap();

        obj.setup_widgets();
        obj
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let self_ = imp::SharMaVidWindow::from_instance(self);

        let (width, height) = self.default_size();

        self_.settings.set_int("window-width", width)?;
        self_.settings.set_int("window-height", height)?;

        self_
            .settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }
    pub fn setup_widgets(&self) {
        let self_ = imp::SharMaVidWindow::from_instance(self);
        self_
            .video_list_model
            .bind_to_list_box(&*self_.video_list, move |v| VideoRow::new(v).upcast());

        /*self_
        .video_page
        .connect_local("view-channel", false, {
            let stack = stack.clone();
            let channel_page = self_.channel_page.clone();
            move |channel_id| {
                stack.clone().set_visible_child_name("channel");
                channel_page.set_channel(channel_id[1].get().unwrap());
                None
            }
        })
        .unwrap();*/
        self_.back_btn.connect_clicked({
            let stack = self_.stack.clone();
            move |_| {
                stack.clone().set_visible_child_name("home");
                let model = stack.pages();
                if let Some(prev_page) = model.item(model.n_items() - 2) {
                    stack.set_visible_child(
                        &prev_page
                            .downcast::<gtk::StackPage>()
                            .expect("Not a gtk::StackPage")
                            .child(),
                    );
                    if let Some(last_page) = model.item(model.n_items() - 1) {
                        stack.remove(
                            &last_page
                                .downcast::<gtk::StackPage>()
                                .expect("Not a gtk::StackPage")
                                .child(),
                        );
                    }
                }
            }
        });

        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        self_.action_pusher.set(sender).unwrap();

        let action_pusher = self.action_pusher();
        self_.video_list.connect_row_activated(move |_, row| {
            let child: VideoRow = row.clone().downcast().unwrap();
            action_pusher
                .send(Action::ShowVideo(child.video()))
                .unwrap();
        });
        let stack = self_.stack.clone();
        let action_pusher = self.action_pusher();
        let client = self_.client.get().unwrap().clone();
        receiver.attach(None, move |action| {
            match action {
                Action::ShowVideo(v) => {
                    let page = VideoPage::new(client.clone(), v, action_pusher.clone());
                    stack.add_child(&page);
                    stack.set_visible_child(&page);
                }
                Action::ShowChannelByID(c_id) => {
                    let page = ChannelPage::new(client.clone(), action_pusher.clone());
                    page.set_channel(c_id);
                    stack.add_child(&page);
                    stack.set_visible_child(&page);
                }
                _ => panic!("PANIC"),
            }
            Continue(true)
        });
    }

    pub fn action_pusher(&self) -> glib::Sender<Action> {
        let self_ = imp::SharMaVidWindow::from_instance(self);
        self_.action_pusher.get().unwrap().clone()
    }
    fn load_window_size(&self) {
        let self_ = imp::SharMaVidWindow::from_instance(self);

        let width = self_.settings.int("window-width");
        let height = self_.settings.int("window-height");
        let is_maximized = self_.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }
    pub fn load_popular(&self) {
        let self_ = imp::SharMaVidWindow::from_instance(&self);
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
