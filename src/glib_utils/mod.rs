mod any_list_model;
pub use any_list_model::*;

use futures::channel::mpsc;
use futures::prelude::*;
use futures::task::{Context, Poll};
use glib::prelude::*;
use gtk::glib;
use std::cell::Cell;
use std::pin::Pin;

pub struct EvStream<T> {
    object: glib::WeakRef<glib::object::Object>,
    signal_id: Cell<Option<glib::SignalHandlerId>>,
    receiver: mpsc::UnboundedReceiver<T>,
}

impl<T> EvStream<T> {
    pub fn new(
        object: glib::WeakRef<glib::object::Object>,
        signal_id: glib::SignalHandlerId,
        receiver: mpsc::UnboundedReceiver<T>,
    ) -> Self {
        Self {
            object,
            signal_id: Cell::new(Some(signal_id)),
            receiver,
        }
    }
}

impl<T> futures::stream::Stream for EvStream<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::get_mut(self).receiver.poll_next_unpin(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.receiver.size_hint()
    }
}

impl<T> std::ops::Drop for EvStream<T> {
    fn drop(&mut self) {
        self.object
            .upgrade()
            .map(|obj| obj.disconnect(self.signal_id.take().unwrap()));
    }
}

#[macro_export]
macro_rules! ev_stream {
    ($this:expr, $event:ident, | $($x:ident),* | $cloning_body:expr) => {
        {
            let (s, r) = ::futures::channel::mpsc::unbounded();
            let signal_id = paste::expr!($this.[<connect_ $event>](move |$($x,)*| {
                let args = $cloning_body;
                s.unbounded_send(args).expect("sending value in ev_stream");
            }));
            crate::EvStream::new($this.clone().upcast::<glib::Object>().downgrade(), signal_id, r)
        }
    };
    ($this:expr, $event:ident, | $($x:ident),* |) => {
        crate::ev_stream!($this, $event, | $($x),* | {
            ($($x.clone()),*) // tuple with cloned elements
        })
    }
}
