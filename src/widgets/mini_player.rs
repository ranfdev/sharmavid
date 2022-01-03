use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::invidious::core::FullVideo;
use crate::widgets::Thumbnail;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/mini_player.ui")]
    pub struct MiniPlayer {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub author_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub thumbnail: TemplateChild<Thumbnail>,
    }

    impl Default for MiniPlayer {
        fn default() -> Self {
            Self {
                title: TemplateChild::default(),
                author_name: TemplateChild::default(),
                thumbnail: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MiniPlayer {
        const NAME: &'static str = "MiniPlayer";
        type Type = super::MiniPlayer;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl WidgetImpl for MiniPlayer {}
    impl ObjectImpl for MiniPlayer {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            obj.prepare_widgets();
        }
    }
    impl BoxImpl for MiniPlayer {}
}

glib::wrapper! {
    pub struct MiniPlayer(ObjectSubclass<imp::MiniPlayer>)
        @extends gtk::Widget, gtk::Box;
}

impl MiniPlayer {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create MiniPlayer")
    }
    pub fn set_video(&self, mut video: FullVideo) {
        let imp = self.imp();
        video
            .video_thumbnails
            .sort_by(|a, b| a.width.partial_cmp(&b.width).unwrap());
        let best_thumbnail = video.video_thumbnails.last().unwrap();
        imp.title.set_label(&video.title);
        imp.thumbnail.set_href(best_thumbnail.url.clone());

        imp.author_name.set_label(&video.author);
    }
    fn prepare_widgets(&self) {
        let imp = self.imp();

        let ev_controller = gtk::GestureClick::new();
        let miniplayer = self.downgrade();
        ev_controller.connect_local("pressed", false, move |_| {
            miniplayer
                .upgrade()
                .map(|mp| mp.activate_action("win.unminimize-video", None))
                .unwrap()
                .unwrap();
            None
        });
        self.add_controller(&ev_controller);
    }
}
