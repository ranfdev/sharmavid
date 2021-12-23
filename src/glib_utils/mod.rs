mod any_list_model;
pub use any_list_model::*;

#[macro_export]
macro_rules! ev_stream_signalled {
    ($this:expr, $event:ident, $($x:ident),*) => {
        {
            let (s, r) = futures::channel::mpsc::unbounded();
            let signal_id = paste::expr!($this.[<connect_ $event>](move |$($x,)*| {
                let args = ($($x.clone(),)*);
                s.unbounded_send(args).unwrap();
            }));
            (signal_id, r)
        }
    };
}

#[macro_export]
macro_rules! ev_stream {
    ($this:expr, $event:ident, $($x:ident),*) => {
        crate::ev_stream_signalled!($this, $event, $($x),*).1
    };
}
