use crate::glib_utils::RustedListModel;
use crate::invidious::core::TrendingVideo;
use crate::widgets::{Action, RemoteImage, VideoRow};
use crate::Client;
use anyhow::anyhow;
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
        pub video_list_model: RustedListModel<TrendingVideo>,
        pub client: OnceCell<Client>,
        pub action_pusher: OnceCell<glib::Sender<Action>>,
    }

    impl Default for ChannelPage {
        fn default() -> Self {
            Self {
                author_name: TemplateChild::default(),
                author_avatar: TemplateChild::default(),
                banner: TemplateChild::default(),
                sub_count: TemplateChild::default(),
                video_list: TemplateChild::default(),
                video_list_model: RustedListModel::new(),
                client: OnceCell::new(),
                action_pusher: OnceCell::new(),
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
    pub fn new(client: Client, action_pusher: glib::Sender<Action>) -> Self {
        let obj: Self = glib::Object::new(&[]).expect("Failed to create ChannelPage");

        let self_ = imp::ChannelPage::from_instance(&obj);
        self_.client.set(client).unwrap();
        self_.action_pusher.set(action_pusher).unwrap();
        obj.prepare_widgets();
        obj
    }
    pub fn set_client(&self, client: Client) {
        let self_ = imp::ChannelPage::from_instance(&self);
        self_.client.set(client).unwrap();
    }
    fn prepare_widgets(&self) {
        let self_ = imp::ChannelPage::from_instance(self);
        self_
            .video_list_model
            .bind_to_list_box(&*self_.video_list, |v| VideoRow::new(v).upcast());
    }
    async fn load_videos(&self, channel_id: String) -> anyhow::Result<()> {
        let self_ = imp::ChannelPage::from_instance(&self);
        let channel = self_
            .client
            .get()
            .unwrap()
            .channel(&channel_id)
            .await
            .map_err(|e| anyhow!(e))?;
        self_
            .author_avatar
            .set_image_url(channel.author_thumbnails.last().unwrap().url.clone())
            .await;
        self_
            .banner
            .set_image_url(channel.author_banners.first().unwrap().url.clone())
            .await;
        self_.author_name.set_label(&channel.author);
        self_
            .sub_count
            .set_label(&format!("{} subscribers", &channel.sub_count));
        self_.video_list_model.clear();
        self_
            .video_list_model
            .extend(channel.latest_videos.into_iter());
        Ok(())
    }
    pub fn set_channel(&self, channel_id: String) {
        let cloned_self = self.clone();
        glib::MainContext::default().spawn_local(async move {
            cloned_self.load_videos(channel_id).await.unwrap();
        });
    }
}
