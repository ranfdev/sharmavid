use crate::glib_utils::RustedListModel;
use crate::invidious::core::{Comment, TrendingVideo};
use crate::widgets::{RemoteImage, Thumbnail};
use crate::Client;
use glib::subclass::signal::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use libadwaita as adw;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use std::cell::RefCell;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/video_page.ui")]
    pub struct VideoPage {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub views_plus_time: TemplateChild<gtk::Label>,
        #[template_child]
        pub video_player: TemplateChild<gtk::Box>,
        #[template_child]
        pub author_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub author_avatar: TemplateChild<adw::Avatar>,
        #[template_child]
        pub view_channel_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub description: TemplateChild<gtk::Label>,
        #[template_child]
        pub comments_list: TemplateChild<gtk::ListBox>,
        pub comments_model: RustedListModel<Comment>,
        pub thumbnail: Thumbnail,
        pub video: RefCell<TrendingVideo>,
        pub client: OnceCell<Client>,
    }

    impl Default for VideoPage {
        fn default() -> Self {
            Self {
                title: TemplateChild::default(),
                views_plus_time: TemplateChild::default(),
                video_player: TemplateChild::default(),
                author_name: TemplateChild::default(),
                author_avatar: TemplateChild::default(),
                view_channel_btn: TemplateChild::default(),
                description: TemplateChild::default(),
                comments_list: TemplateChild::default(),
                comments_model: RustedListModel::new(),
                thumbnail: Thumbnail::new(None),
                video: RefCell::new(TrendingVideo::default()),
                client: OnceCell::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoPage {
        const NAME: &'static str = "VideoPage";
        type Type = super::VideoPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl WidgetImpl for VideoPage {}
    impl ObjectImpl for VideoPage {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder(
                    // Signal name
                    "view-channel",
                    // Types of the values which will be sent to the signal handler
                    &[String::static_type().into()],
                    // Type of the value the signal handler sends back
                    <()>::static_type().into(),
                )
                .build()]
            });
            SIGNALS.as_ref()
        }
    }
    impl BoxImpl for VideoPage {}
}

glib::wrapper! {
    pub struct VideoPage(ObjectSubclass<imp::VideoPage>)
        @extends gtk::Widget, gtk::Box;
}

impl VideoPage {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create VideoPage");

        obj.prepare_widgets();
        obj
    }
    fn prepare_widgets(&self) {
        let self_ = imp::VideoPage::from_instance(self);
        self_.video_player.append(&self_.thumbnail);
        self_.thumbnail.set_hexpand(true);
        self_
            .comments_model
            .bind_to_list_box(&*self_.comments_list, |c| Self::build_comment(c));
        let cloned_self = self.clone();
        self_.view_channel_btn.connect_clicked(move |_| {
            let self_ = imp::VideoPage::from_instance(&cloned_self);
            let video = self_.video.borrow();
            cloned_self
                .emit_by_name("view-channel", &[&video.author_id])
                .unwrap();
        });
    }
    pub fn set_client(&self, client: Client) {
        let self_ = imp::VideoPage::from_instance(self);
        self_.client.set(client).unwrap();
    }

    pub fn set_video(&self, mut video: TrendingVideo) {
        let self_ = imp::VideoPage::from_instance(&self);
        *self_.video.borrow_mut() = video.clone();
        self_.title.set_label(&video.title);
        self_
            .views_plus_time
            .set_label(&format!("{} views · {}", video.view_count, video.published));

        self_.author_name.set_label(&video.author);
        if let Some(description) = video.description {
            self_.description.set_label(&description);
        }
        video
            .video_thumbnails
            .sort_by(|a, b| a.width.partial_cmp(&b.width).unwrap());
        let best_thumbnail = video.video_thumbnails.last().unwrap();
        self_.thumbnail.set_href(best_thumbnail.url.clone());

        let video_id = video.video_id.clone();
        let thumbnail = self_.thumbnail.clone();
        let comments_model = self_.comments_model.clone();
        let author_name = self_.author_name.clone();
        let author_avatar = self_.author_avatar.clone();
        let description = self_.description.clone();
        let client = self_.client.get().unwrap().clone();
        glib::MainContext::default().spawn_local_with_priority(glib::PRIORITY_LOW, async move {
            comments_model.clear();
            let comments = client.comments(&video_id).await.unwrap();
            comments_model.extend(comments.comments.into_iter());

            // Needed because the Video doesn't contain the entire description and author data.
            let video = client.video(&video.video_id).await.unwrap();
            description.set_label(&video.description);

            author_name.set_label(&video.author);
            author_avatar
                .set_image_url(video.author_thumbnails.first().unwrap().url.clone())
                .await;

            let ev_controller = gtk::GestureClick::new();
            ev_controller
                .connect("pressed", false, move |_| {
                    gtk::show_uri(
                        None::<&gtk::Window>,
                        &video.adaptive_formats.first().unwrap().url,
                        0,
                    );
                    None
                })
                .unwrap();
            thumbnail.add_controller(&ev_controller);
        })
    }
    pub fn build_comment(comment: Comment) -> gtk::Widget {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_top(4);
        hbox.set_margin_bottom(4);
        hbox.set_margin_start(4);
        hbox.set_margin_end(4);
        let avatar = adw::Avatar::new(32, Some(&comment.author), true);
        avatar.set_valign(gtk::Align::Start);
        hbox.append(&avatar);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
        hbox.append(&vbox);

        let author_name_label = gtk::Label::new(Some(&comment.author));
        author_name_label.set_xalign(0.0);
        author_name_label.set_halign(gtk::Align::Start);
        author_name_label.set_wrap(true);
        author_name_label.set_wrap_mode(pango::WrapMode::WordChar);
        author_name_label.add_css_class("heading");
        vbox.append(&author_name_label);

        let comment_label = gtk::Label::new(Some(&comment.content));
        comment_label.set_xalign(0.0);
        comment_label.set_halign(gtk::Align::Start);
        comment_label.set_wrap(true);
        comment_label.set_wrap_mode(pango::WrapMode::WordChar);
        vbox.append(&comment_label);

        let comment_clone = comment.clone();
        glib::MainContext::default().spawn_local(async move {
            avatar
                .set_image_url(comment_clone.author_thumbnails.first().unwrap().url.clone())
                .await;
        });

        hbox.upcast()
    }
}
