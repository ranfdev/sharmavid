use crate::widgets::RemoteImage;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

mod imp {
    use super::*;

    use gtk::CompositeTemplate;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/ranfdev/SharMaVid/ui/thumbnail.ui")]
    pub struct Thumbnail {
        pub href: RefCell<String>,
        #[template_child]
        pub img: TemplateChild<gtk::Picture>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
    }

    impl Default for Thumbnail {
        fn default() -> Self {
            Self {
                href: RefCell::new(String::new()),
                img: TemplateChild::default(),
                stack: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Thumbnail {
        const NAME: &'static str = "Thumbnail";
        type Type = super::Thumbnail;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl WidgetImpl for Thumbnail {}
    impl ObjectImpl for Thumbnail {}
    impl BoxImpl for Thumbnail {}
}

glib::wrapper! {
    pub struct Thumbnail(ObjectSubclass<imp::Thumbnail>)
        @extends gtk::Widget, gtk::Box;
}

impl Thumbnail {
    pub fn new(href: Option<String>) -> Self {
        let obj: Thumbnail = glib::Object::new(&[]).expect("Failed to create Thumbnail");
        if let Some(href) = href {
            obj.set_href(href);
        }
        obj
    }

    pub fn set_href(&self, href: String) {
        let self_ = self.impl_();
        *self_.href.borrow_mut() = href.clone();

        let img = self_.img.clone();
        let stack = self_.stack.clone();
        stack.set_visible_child_name("placeholder");
        glib::MainContext::default().spawn_local_with_priority(
            glib::source::PRIORITY_LOW,
            async move {
                img.set_image_url_future(href).await;
                stack.set_visible_child_name("img");
            },
        );
    }
}
