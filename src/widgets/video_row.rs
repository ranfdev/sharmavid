use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

use crate::invidious::core::*;
use crate::widgets::Thumbnail;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/video_row.ui")]
    pub struct VideoRow {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub author: TemplateChild<gtk::Label>,
        #[template_child]
        pub views: TemplateChild<gtk::Label>,
        #[template_child]
        pub thumbnail: TemplateChild<Thumbnail>,
        pub video: RefCell<TrendingVideo>,
    }

    impl Default for VideoRow {
        fn default() -> Self {
            Self {
                title: TemplateChild::default(),
                views: TemplateChild::default(),
                author: TemplateChild::default(),
                thumbnail: TemplateChild::default(),
                video: RefCell::new(TrendingVideo::default()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoRow {
        const NAME: &'static str = "VideoRow";
        type Type = super::VideoRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl WidgetImpl for VideoRow {}
    impl ObjectImpl for VideoRow {}
    impl ListBoxRowImpl for VideoRow {}
}

glib::wrapper! {
    pub struct VideoRow(ObjectSubclass<imp::VideoRow>)
        @extends gtk::Widget, gtk::ListBoxRow, gtk::Actionable;
}

impl VideoRow {
    pub fn new(video: TrendingVideo) -> Self {
        let obj: VideoRow = glib::Object::new(&[]).expect("Failed to create VideoRow");
        obj.prepare_widgets();
        obj.set_video(video);
        obj
    }
    pub fn set_video(&self, mut video: TrendingVideo) {
        let imp = self.imp();
        video
            .video_thumbnails
            .sort_by(|a, b| a.width.partial_cmp(&b.width).unwrap());
        *imp.video.borrow_mut() = video.clone();
        let thumbnail_url = video.video_thumbnails.last().unwrap().url.clone();
        imp.thumbnail.set_href(thumbnail_url);
        imp.title.set_label(&video.title);
        imp.author.set_label(&video.author);
        imp.views.set_label(&format!("{} views", &video.view_count));
    }
    fn prepare_widgets(&self) {
        let imp = self.imp();
        imp.thumbnail.set_width_request(160);
        imp.thumbnail.set_height_request(90);
    }
    pub fn video(&self) -> TrendingVideo {
        let imp = self.imp();
        imp.video.borrow().clone()
    }
}
