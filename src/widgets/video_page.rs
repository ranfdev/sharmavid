use crate::glib_utils::{RustedListStore, RustedListBox};
use crate::invidious::core::{Comment, FullVideo};
use crate::widgets::{RemoteImageExt, Thumbnail};
use crate::Client;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use libadwaita as adw;
use once_cell::sync::OnceCell;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/video_page.ui")]
    pub struct VideoPage {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub miniplayer_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub views_plus_time: TemplateChild<gtk::Label>,
        #[template_child]
        pub video_player: TemplateChild<gtk::Box>,
        #[template_child]
        pub author_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub miniplayer_author: TemplateChild<gtk::Label>,
        #[template_child]
        pub author_avatar: TemplateChild<adw::Avatar>,
        #[template_child]
        pub view_channel_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub description: TemplateChild<gtk::Label>,
        #[template_child]
        pub comments_list: TemplateChild<gtk::ListBox>,
        pub comments_model: RustedListStore<Comment>,
        pub thumbnail: Thumbnail,
        #[template_child]
        pub miniplayer_thumbnail: TemplateChild<Thumbnail>,
        pub video: OnceCell<FullVideo>,
        pub client: OnceCell<Client>,
    }

    impl Default for VideoPage {
        fn default() -> Self {
            Self {
                title: TemplateChild::default(),
                miniplayer_title: TemplateChild::default(),
                views_plus_time: TemplateChild::default(),
                video_player: TemplateChild::default(),
                author_name: TemplateChild::default(),
                miniplayer_author: TemplateChild::default(),
                author_avatar: TemplateChild::default(),
                view_channel_btn: TemplateChild::default(),
                description: TemplateChild::default(),
                comments_list: TemplateChild::default(),
                comments_model: RustedListStore::new(),
                thumbnail: Thumbnail::new(None),
                miniplayer_thumbnail: TemplateChild::default(),
                video: OnceCell::default(),
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
    impl ObjectImpl for VideoPage {}
    impl BoxImpl for VideoPage {}
}

glib::wrapper! {
    pub struct VideoPage(ObjectSubclass<imp::VideoPage>)
        @extends gtk::Widget, gtk::Box;
}

impl VideoPage {
    pub fn new(client: Client) -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create VideoPage");
        obj.set_client(client);
        obj.prepare_widgets();
        obj
    }
    fn prepare_widgets(&self) {
        let self_ = self.impl_();
        self_.video_player.append(&self_.thumbnail);
        self_.thumbnail.set_hexpand(true);
        self_.thumbnail.set_height_request(200);
        self_.comments_list
            .bind_rusted_model(&self_.comments_model, |c| Self::build_comment(c.clone()));
    }
    pub fn set_client(&self, client: Client) {
        let self_ = self.impl_();
        self_.client.set(client).unwrap();
    }
    pub(super) fn set_video(&self, mut video: FullVideo) {
        let self_ = self.impl_();
        self_.video.set(video.clone()).unwrap();
        self_.title.set_label(&video.title);
        self_.miniplayer_title.set_label(&video.title);
        self_
            .views_plus_time
            .set_label(&format!("{} views · {}", video.view_count, video.published));
        self_.author_name.set_label(&video.author);
        self_.description.set_label(&video.description);
        self_
            .view_channel_btn
            .set_action_target_value(Some(&video.author_id.to_variant()));
        self_
            .view_channel_btn
            .set_action_name(Some("win.view-channel"));
        video
            .video_thumbnails
            .sort_by(|a, b| a.width.partial_cmp(&b.width).unwrap());
        let best_thumbnail = video.video_thumbnails.last().unwrap();
        self_.thumbnail.set_href(best_thumbnail.url.clone());
        self_
            .miniplayer_thumbnail
            .set_href(best_thumbnail.url.clone());

        self_.miniplayer_author.set_label(&video.author);
        self_.author_name.set_label(&video.author);

        let ev_controller = gtk::GestureClick::new();
        ev_controller.connect("pressed", false, move |_| {
            gtk::show_uri(
                None::<&gtk::Window>,
                &video.adaptive_formats.first().unwrap().url,
                0,
            );
            None
        });
        self_.thumbnail.add_controller(&ev_controller);
        self_
            .author_avatar
            .set_image_url(video.author_thumbnails.first().unwrap().url.clone());

        let video_id = video.video_id.clone();
        let comments_model = self_.comments_model.clone();
        let client = self_.client.get().unwrap().clone();
        glib::MainContext::default().spawn_local_with_priority(glib::PRIORITY_LOW, async move {
            comments_model.clear();
            let comments = client.comments(&video_id).await.unwrap();
            comments_model.extend(comments.comments.into_iter());
        });
    }
    pub fn build_comment(comment: Comment) -> gtk::Widget {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);
        hbox.set_margin_start(8);
        hbox.set_margin_end(8);

        let avatar = adw::Avatar::new(32, Some(&comment.author), true);
        avatar.set_image_url(comment.author_thumbnails.first().unwrap().url.clone());
        let avatar_btn = gtk::Button::builder()
            .child(&avatar)
            .valign(gtk::Align::Start)
            .action_name("win.view-channel")
            .action_target(&comment.author_id.to_variant())
            .css_classes(vec!["flat".to_string(), "circular".to_string()])
            .build();
        hbox.append(&avatar_btn);

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

        hbox.upcast()
    }
}
