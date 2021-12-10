use gtk::glib;
use gtk::subclass::prelude::*;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct AnyGObject {
        pub item: RefCell<Option<Box<dyn Any>>>,
    }
    impl Default for AnyGObject {
        fn default() -> Self {
            Self {
                item: RefCell::new(None),
            }
        }
    }
    #[glib::object_subclass]
    impl ObjectSubclass for AnyGObject {
        const NAME: &'static str = "AnyGObject";
        type Type = super::AnyGObject;
        type ParentType = glib::Object;
    }
    impl ObjectImpl for AnyGObject {}
}

glib::wrapper! {
    pub struct AnyGObject(ObjectSubclass<imp::AnyGObject>);
}

impl AnyGObject {
    pub fn new(item: Box<dyn Any>) -> Self {
        let obj: AnyGObject = glib::Object::new(&[]).expect("Failed to create AnyGObject");

        *obj.impl_().item.borrow_mut() = Some(item);
        obj
    }
    pub fn borrow<'a, T: 'static>(&'a self) -> Ref<'a, T> {
        Ref::map(self.impl_().item.borrow(), |item| {
            item.as_ref().unwrap().downcast_ref::<T>().unwrap()
        })
    }
    pub fn borrow_mut<'a, T: 'static>(&'a mut self) -> RefMut<'a, T> {
        RefMut::map(self.impl_().item.borrow_mut(), |item| {
            item.as_mut().unwrap().downcast_mut::<T>().unwrap()
        })
    }
}
