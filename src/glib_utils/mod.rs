mod any_list_model;
pub use any_list_model::*;

#[macro_export]
macro_rules! ev_stream_signalled {
    ($this:expr, $event:ident, | $($x:ident),* | $cloning_body:expr) => {
        {
            let (s, r) = futures::channel::mpsc::unbounded();
            let signal_id = paste::expr!($this.[<connect_ $event>](move |$($x,)*| {
                let args = $cloning_body;
                s.unbounded_send(args).unwrap();
            }));
            (signal_id, r)
        }
    };
    ($this:expr, $event:ident, | $($x:ident),* |) => {
        crate::ev_stream_signalled!($this, $event, | $($x),* | {
            ($($x.clone()),*) // tuple with cloned elements
        })
    }
}

#[macro_export]
macro_rules! ev_stream {
    ($this:expr, $event:ident, | $($x:ident),* | $cloning_body:expr) => {
        crate::ev_stream_signalled!($this, $event, | $($x),* | $cloning_body).1
    };
    ($this:expr, $event:ident, | $($x:ident),* |) => {
        crate::ev_stream_signalled!($this, $event, | $($x),* |).1
    }
}
