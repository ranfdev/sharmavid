use crate::glib_utils::AnyGobject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use log::warn;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, Default)]
    pub struct AnyListModel {
        pub vec: RefCell<Vec<glib::Object>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AnyListModel {
        const NAME: &'static str = "AnyListModel";
        type ParentType = glib::Object;
        type Type = super::AnyListModel;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for AnyListModel {}

    impl ListModelImpl for AnyListModel {
        fn item_type(&self, _list_model: &Self::Type) -> glib::Type {
            glib::Object::static_type()
        }
        fn n_items(&self, _list_model: &Self::Type) -> u32 {
            self.vec.borrow().len() as u32
        }
        fn item(&self, _list_model: &Self::Type, position: u32) -> Option<glib::Object> {
            self.vec
                .borrow()
                .get(position as usize)
                .map(|o| o.clone().upcast::<glib::Object>())
        }
    }
}

glib::wrapper! {
    pub struct AnyListModel(ObjectSubclass<imp::AnyListModel>) @implements gio::ListModel;
}

impl AnyListModel {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).unwrap()
    }

    pub fn add_item(&self, item: glib::Object) {
        let self_ = self.impl_();

        // Own scope to avoid "already mutably borrowed: BorrowError"
        let pos = {
            let mut data = self_.vec.borrow_mut();
            data.push(item.clone());
            (data.len() - 1) as u32
        };

        self.items_changed(pos, 0, 1);
    }

    pub fn clear(&self) {
        let len = self.n_items();
        self.impl_().vec.borrow_mut().clear();
        self.items_changed(0, len, 0);
    }
    pub fn extend(&self, iter: impl Iterator<Item = glib::Object>) {
        let self_ = self.impl_();

        let (pos, c) = {
            let mut data = self_.vec.borrow_mut();
            let plen = data.len();
            data.extend(iter);
            (plen as u32, (data.len() - plen) as u32)
        };
        self.items_changed(pos, 0, c);
    }
}

#[derive(Debug, Clone)]
pub struct RustedListModel<T> {
    list_model: AnyListModel,
    phantom: std::marker::PhantomData<T>,
}

// TODO: the Clone requirement could be relaxed and performance improved
impl<T: 'static + Clone> RustedListModel<T> {
    pub fn new() -> Self {
        Self {
            list_model: AnyListModel::new(),
            phantom: std::marker::PhantomData,
        }
    }
    pub fn add_item(&self, item: T) {
        self.list_model
            .add_item(AnyGobject::new(Box::new(item)).upcast())
    }
    pub fn clear(&self) {
        self.list_model.clear();
    }
    pub fn extend<I: Iterator<Item = T>>(&self, iter: I) {
        self.list_model
            .extend(iter.map(|item| AnyGobject::new(Box::new(item)).upcast()));
    }
    pub fn as_gio(&self) -> gio::ListModel {
        self.list_model.clone().upcast()
    }
    pub fn bind_to_list_box<L: glib::object::IsA<gtk::ListBox>>(
        &self,
        list_box: &L,
        bind_func: impl Fn(T) -> gtk::Widget + 'static,
    ) {
        let list_box: gtk::ListBox = list_box.clone().upcast();
        list_box.bind_model(Some(&self.as_gio()), move |v|
                bind_func(
                    v.clone().downcast::<AnyGobject>()
                        .expect("Couldn't downcast gobject to rust type. Check if the binded model is a RustedListModel")
                        .item::<T>()
                        .expect("Couldn't downcast gobject to rust type. Check if the binded model is a RustedListModel")
                ))
    }
}
