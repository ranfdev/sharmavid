use crate::glib_utils::{RustedListBox, RustedListStore};
use crate::invidious::core::{Channel, TrendingVideo};
use crate::widgets::{RemoteImageExt, VideoRow};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use libadwaita as adw;
use once_cell::sync::OnceCell;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/channel_page.ui")]
    pub struct ChannelPage {
        #[template_child]
        pub author_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub author_avatar: TemplateChild<adw::Avatar>,
        #[template_child]
        pub banner: TemplateChild<gtk::Picture>,
        #[template_child]
        pub sub_count: TemplateChild<gtk::Label>,
        #[template_child]
        pub video_list: TemplateChild<gtk::ListBox>,
        pub video_list_model: RustedListStore<TrendingVideo>,
    }

    impl Default for ChannelPage {
        fn default() -> Self {
            Self {
                author_name: TemplateChild::default(),
                author_avatar: TemplateChild::default(),
                banner: TemplateChild::default(),
                sub_count: TemplateChild::default(),
                video_list: TemplateChild::default(),
                video_list_model: RustedListStore::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannelPage {
        const NAME: &'static str = "ChannelPage";
        type Type = super::ChannelPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl WidgetImpl for ChannelPage {}
    impl ObjectImpl for ChannelPage {}
    impl BoxImpl for ChannelPage {}
}

glib::wrapper! {
    pub struct ChannelPage(ObjectSubclass<imp::ChannelPage>)
        @extends gtk::Widget, gtk::Box;
}

impl ChannelPage {
    pub fn new() -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create ChannelPage");

        obj.prepare_widgets();
        obj
    }
    fn prepare_widgets(&self) {
        let imp = self.imp();
        imp.video_list
            .bind_rusted_model(&imp.video_list_model, |v| VideoRow::new(v.clone()).upcast());
        imp.video_list.connect_row_activated(|_, row| {
            let row: VideoRow = row.clone().downcast().unwrap();
            row.activate_action("win.view-video", Some(&row.video().video_id.to_variant()))
                .unwrap();
        });
    }
    pub fn set_channel(&self, channel: Channel) {
        let imp = self.imp();
        channel
            .author_thumbnails
            .last()
            .map(|image| imp.author_avatar.set_image_url(image.url.clone()));

        channel
            .author_banners
            .first()
            .map(|image| imp.banner.set_image_url(image.url.clone()));

        imp.author_name.set_label(&channel.author);
        imp.sub_count
            .set_label(&format!("{} subscribers", &channel.sub_count));
        imp.video_list_model.clear();
        imp.video_list_model
            .extend(channel.latest_videos.into_iter());
    }
}
