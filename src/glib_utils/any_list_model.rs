use gtk::prelude::*;
use gtk::{gio, glib};
use glib::BoxedAnyObject;
use std::marker::PhantomData;


#[derive(Debug, Clone)]
pub struct RustedListStore<T>(gio::ListStore, PhantomData<T>);

// TODO: the Clone requirement could be relaxed and performance improved
impl<T: 'static + Clone> RustedListStore<T> {
    pub fn new() -> Self {
        Self(gio::ListStore::new(BoxedAnyObject::static_type()), std::marker::PhantomData)
    }
    pub fn add_item(&self, item: T) {
        self.0
            .append(&BoxedAnyObject::new(item))
    }
    pub fn clear(&self) {
        self.0.remove_all();
    }
    pub fn extend<I: Iterator<Item = T>>(&self, iter: I) {
        self.0.clone().extend(iter.map(BoxedAnyObject::new));
    }
}

impl<T: Clone> std::ops::Deref for RustedListStore<T> {
    type Target = gio::ListStore;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait RustedListBox {
    fn bind_rusted_model<T: 'static + Clone>(
        &self,
        store: &RustedListStore<T>,
        bind_func: impl Fn(&T) -> gtk::Widget + 'static
    );
}

impl RustedListBox for gtk::ListBox {
    fn bind_rusted_model<T: 'static + Clone>(
        &self,
        store: &RustedListStore<T>,
        bind_func: impl Fn(&T) -> gtk::Widget + 'static
    ) {
        self.bind_model(Some(&**store), move |v|
                bind_func(
                    &v.clone().downcast::<BoxedAnyObject>()
                        .expect("Couldn't downcast gobject to rust type. Check if the binded model is a RustedListModel")
                        .borrow::<T>()
                ))
    }
}
