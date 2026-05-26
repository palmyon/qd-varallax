use std::sync::{Arc, Mutex};

use crate::types::genelational_vector::{VxGenIndex, VxGenVector};


pub(crate) trait VxSignalMsg: Send + 'static {}
impl<T: Send + 'static> VxSignalMsg for T {}

pub(crate) struct VxSignal<Sender: ?Sized, M: VxSignalMsg> {
	handler: Arc<Mutex<VxGenVector<Box<dyn Fn(&mut Sender, &M) + Send + Sync>>>>,
}

impl<Sender: ?Sized, Message: VxSignalMsg> VxSignal<Sender, Message> {
	pub fn new() -> Self {
		Self { handler: Arc::new(Mutex::new(VxGenVector::new())) }
	}

	#[inline(always)]
	pub fn emit(&self, sender: &mut Sender, msg: &Message) {
		let h = self.handler.clone();
		let lock = h.lock().unwrap();
		for c in lock.iter() {
			c(sender, &msg);
		}
	}

	pub(crate) fn internal_connect(&self, f: Box<dyn Fn(&mut Sender, &Message) + Send + Sync>) -> VxGenIndex {
		self.handler.lock().unwrap().insert(f)
	}
	pub(crate) fn internal_disconnect(&self, id: VxGenIndex) -> bool {
		self.handler.lock().unwrap().remove(id).is_some()
	}
	pub(crate) fn internal_clear(&self) -> usize {
		let mut lock = self.handler.lock().unwrap();
		let len = lock.len();
		lock.clear();
		len
	}
}

/// ## QD-Varallax> Macros> VxSignal!
/// # Usage
/// ```no_run
/// // Defines a `VxSignal` struct named `FooSignal` that returns no value when emitted.
/// vx_signal!(pub struct FooSignal >> ());
/// // Defines a `VxSignal` struct named `BarSignal` that returns a `bool` when emitted.
/// vx_signal!(pub struct BarSignal >> bool);
/// ```
#[macro_export]
macro_rules! vx_signal {
	($vis:vis struct $name:ident >> $msg:ty) => {
		$vis struct $name<Sender: ?Sized> {
			inner: std::sync::Arc<$crate::core::signal::VxSignal<Sender, $msg>>,
			_marker: std::marker::PhantomData<Sender>,
		}

		impl<Sender: ?Sized> Clone for $name<Sender> {
			fn clone(&self) -> Self {
				Self {
					inner: self.inner.clone(),
					_marker: std::marker::PhantomData,
				}
			}
		}

		impl<Sender: ?Sized> $name<Sender> {
			pub fn new() -> Self {
				Self {
					inner: std::sync::Arc::new($crate::core::signal::VxSignal::new()),
					_marker: std::marker::PhantomData,
				}
			}
			#[inline(always)]
			pub fn emit(&self, sender: &mut Sender, msg: &$msg) {
				self.inner.emit(sender, &msg);
			}
			pub fn connect(&self, f: Box<dyn Fn(&mut Sender, &$msg) + Send + Sync>) -> $crate::types::genelational_vector::VxGenIndex {
				self.inner.internal_connect(f)
			}
			pub fn disconnect(&self, id: crate::types::genelational_vector::VxGenIndex) -> bool {
				self.inner.internal_disconnect(id)
			}
			pub fn clear(&self) -> usize {
				self.inner.internal_clear()
			}
		}
	};
}


#[macro_export]
macro_rules! vx_connect {
    ($signal:expr, move |$owner:pat_param, $arg:pat_param| $body:expr) => {
        $signal.connect(Box::new(move |$owner, $arg| $body))
    };
    ($signal:expr, |$owner:pat_param, $arg:pat_param| $body:expr) => {
        $signal.connect(Box::new(|$owner, $arg| $body))
    };
}