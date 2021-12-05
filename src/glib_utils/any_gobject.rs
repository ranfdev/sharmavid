use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct AnyGobject {
        pub item: RefCell<Box<dyn std::any::Any>>,
    }
    impl Default for AnyGobject {
        fn default() -> Self {
            let opt: Option<u8> = None;
            Self {
                item: RefCell::new(Box::new(opt)),
            }
        }
    }
    #[glib::object_subclass]
    impl ObjectSubclass for AnyGobject {
        const NAME: &'static str = "AnyGobject";
        type Type = super::AnyGobject;
        type ParentType = glib::Object;
    }
    impl ObjectImpl for AnyGobject {}
}

glib::wrapper! {
    pub struct AnyGobject(ObjectSubclass<imp::AnyGobject>);
}

impl AnyGobject {
    pub fn new(item: Box<dyn std::any::Any>) -> Self {
        let obj: AnyGobject = glib::Object::new(&[]).expect("Failed to create AnyGobject");

        let self_ = imp::AnyGobject::from_instance(&obj);
        *self_.item.borrow_mut() = item;
        obj
    }
    pub fn item<T: 'static + Clone>(&self) -> Option<T> {
        let self_ = imp::AnyGobject::from_instance(self);
        self_
            .item
            .borrow()
            .downcast_ref::<T>()
            .map(|item| item.clone())
    }
}
