use crate::ev_stream;
use crate::glib_utils::{RustedListBox, RustedListStore};
use crate::invidious::core::{Comment, CommentsParams, FullVideo};
use crate::widgets::{MiniPlayer, RemoteImageExt, Thumbnail};
use crate::{ctx, Client};
use futures::prelude::*;
use futures::task::LocalSpawnExt;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, pango};
use libadwaita as adw;
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

        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        pub comments_model: RustedListStore<Comment>,
        pub thumbnail: Thumbnail,
        pub video: RefCell<Option<FullVideo>>,
        pub async_handle: RefCell<Option<future::RemoteHandle<()>>>,
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
                scrolled_window: TemplateChild::default(),

                comments_model: RustedListStore::new(),
                thumbnail: Thumbnail::new(None),
                video: RefCell::default(),
                async_handle: RefCell::default(),
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.prepare_widgets();
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
        glib::Object::new(&[]).expect("Failed to create VideoPage")
    }
    fn prepare_widgets(&self) {
        let self_ = self.impl_();
        self_.video_player.append(&self_.thumbnail);
        self_.thumbnail.set_hexpand(true);
        self_.thumbnail.set_height_request(200);
        self_
            .comments_list
            .bind_rusted_model(&self_.comments_model, |c| Self::build_comment(c.clone()));

        let ev_controller = gtk::GestureClick::new();
        let this = self.downgrade();
        ev_controller.connect_local("pressed", false, move |_| {
            gtk::show_uri(
                None::<&gtk::Window>,
                &this
                    .upgrade()
                    .unwrap()
                    .impl_()
                    .video
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .adaptive_formats
                    .first()
                    .unwrap()
                    .url,
                0,
            );
            None
        });
        self_.thumbnail.add_controller(&ev_controller);

        let ev_controller = gtk::GestureClick::new();
    }
    pub(super) fn set_video(&self, mut video: FullVideo) {
        let self_ = self.impl_();
        self_.video.replace(Some(video.clone()));
        self_.title.set_label(&video.title);

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

        self_.author_name.set_label(&video.author);

        self_
            .author_avatar
            .set_image_url(video.author_thumbnails.first().unwrap().url.clone());

        let video_id = video.video_id.clone();
        let comments_model = self_.comments_model.clone();

        self_.comments_model.clear();
        let comments_params = CommentsParams::default();
        let edge_reached_evs = ev_stream!(self_.scrolled_window, edge_reached, |target, edge|)
            .filter(|(_, edge)| future::ready(*edge == gtk::PositionType::Bottom))
            .map(|_| ());
        let comments_stream = stream::once(async move { () })
            .chain(edge_reached_evs)
            .zip(Client::global().comments(video_id, comments_params))
            .filter_map(|(_, c)| future::ready(c.ok()));
        let comments_loading_effect = comments_stream.for_each(move |comments| {
            future::ready(comments_model.extend(comments.comments.into_iter()))
        });

        let handle = ctx().spawn_local_with_handle(comments_loading_effect).ok();
        self_.async_handle.replace(handle);
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
