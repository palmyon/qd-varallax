use std::{
	cell::RefCell,
	marker::PhantomData,
	ops::Deref,
	rc::Rc,
	sync::{
		Arc,
		Mutex
	}
};

use crate::types::gen_vector::{
	VxGenIndex,
	VxGenVector
};

pub trait VxSignalPolicy {
	type VxSignalContainer<T>: Clone;
	type VxSignalHandler<H: ?Sized>: Clone + Deref<Target = H>;
	type FnClosure<Sender: ?Sized, M>: ?Sized + Fn(&mut Sender, &M);
	fn new<T>(val: T) -> Self::VxSignalContainer<T> ;
	fn new_handler<H: ?Sized>(val: Box<H>) -> Self::VxSignalHandler<H>;
	fn with_mut<T, R>(
		container: &Self::VxSignalContainer<T>,
		f: impl FnOnce(&mut T) -> R
	) -> R;
}

pub struct VxLocalSignalPolicy;
impl VxSignalPolicy for VxLocalSignalPolicy {
	type VxSignalContainer<T> = Rc<RefCell<T>>;
	type VxSignalHandler<H: ?Sized> = Rc<H>;
	type FnClosure<Sender: ?Sized, M> = dyn Fn(&mut Sender, &M);
	fn new<T>(val: T) -> Self::VxSignalContainer<T> {
		Rc::new(RefCell::new(val))
	}
	fn new_handler<H: ?Sized>(val: Box<H>) -> Self::VxSignalHandler<H> {
		Rc::from(val)
	}
	fn with_mut<T, R>(
		container: &Self::VxSignalContainer<T>,
		f: impl FnOnce(&mut T) -> R
	) -> R {
		f(&mut container.borrow_mut())
	}
}

pub struct VxSharedSignalPolicy;
impl VxSignalPolicy for VxSharedSignalPolicy {
	type VxSignalContainer<T> = Arc<Mutex<T>>;
	type VxSignalHandler<H: ?Sized> = Arc<H>;
	type FnClosure<Sender: ?Sized, M> = dyn Fn(&mut Sender, &M) + Send + Sync;
	fn new<T>(val: T) -> Self::VxSignalContainer<T> {
		Arc::new(Mutex::new(val))
	}
	fn new_handler<H: ?Sized>(val: Box<H>) -> Self::VxSignalHandler<H> {
		Arc::from(val)
	}
	fn with_mut<T, R>(
		container: &Self::VxSignalContainer<T>,
		f: impl FnOnce(&mut T) -> R
	) -> R {
		f(&mut container.lock()
			.expect("VxSignal[Shared]> CrititalError: Failed to lock"))
	}
}

pub struct VxSignal
<Sender: ?Sized, M: 'static, S: VxSignalPolicy = VxLocalSignalPolicy> {
	handler: S::VxSignalContainer<VxGenVector<S::VxSignalHandler<S::FnClosure<Sender, M>>>>,
	_marker: PhantomData<S>,
}
impl<Sender: ?Sized, M: 'static, S: VxSignalPolicy> Clone for VxSignal<Sender, M, S> {
	fn clone(&self) -> Self {
		Self {
			handler: self.handler.clone(),
			_marker: PhantomData
		}
	}
}
impl<Sender: ?Sized, M: 'static, S: VxSignalPolicy> VxSignal<Sender, M, S> {
	pub fn new() -> Self {
		Self {
			handler: S::new(VxGenVector::new()),
			_marker: PhantomData,
		}
	}
	#[inline(always)]
	pub fn emit(&self, sender: &mut Sender, msg: &M) {
		let snapshot = S::with_mut(&self.handler, |v| {
			v.iter().cloned().collect::<Vec<_>>()
		});
		for c in snapshot {
			c(sender, msg)
		}
	}
	pub fn internal_connect(&self, f: Box<S::FnClosure<Sender, M>>) -> VxGenIndex {
		S::with_mut(&self.handler, |v| {
			v.insert(S::new_handler(f))
		})
	}
	pub fn internal_disconnect(&self, id: VxGenIndex) -> bool {
		S::with_mut(
			&self.handler,
			|v| v.remove(id)
		).is_some()
	}
	pub fn internal_clear(&self) -> usize {
		S::with_mut(&self.handler, |v| {
			let len = v.len();
			v.clear();
			len
		})
	}
}

/// ## QD-Varallax> Macros> VxSignal!
/// # Usage
/// ```no_run
/// // Defines a `VxSignal` struct named `FooSignal` that passes no value when emitted.
/// vx_signal!(pub struct FooSignal >> ());
/// // Defines a `VxSignal` struct named `BarSignal` that passes a `bool` when emitted.
/// vx_signal!(pub struct BarSignal >> bool);
/// // Defines a thread-safe 'VxSignal` struct named `FooBarSignal` that passes `(i32, usize)` when emitted.
/// vx_signal!(shared pub struct FooBarSignal >> (i32, usize));
/// ```
#[macro_export]
macro_rules! vx_signal {
	($vis:vis struct $name:ident >> $msg:ty) => {
		$vis struct $name<Sender: ?Sized> {
			inner: $crate::core::signal::VxSignal<Sender, $msg>,
		}

		impl<Sender: ?Sized> Clone for $name<Sender> {
			fn clone(&self) -> Self {
				Self { inner: self.inner.clone() }
			}
		}

		impl<Sender: ?Sized> $name<Sender> {
			pub fn new() -> Self {
				Self {
					inner: $crate::core::signal::VxSignal::new(),
				}
			}
			#[inline(always)]
			pub fn emit(&self, sender: &mut Sender, msg: &$msg) {
				self.inner.emit(sender, msg);
			}
			pub fn connect<F>(
				&self, f: F
			) -> $crate::types::gen_vector::VxGenIndex
			where F: Fn(&mut Sender, &$msg) + 'static
			{
				self.inner.internal_connect(Box::new(f))
			}
			pub fn disconnect(&self, id: $crate::types::gen_vector::VxGenIndex) -> bool {
				self.inner.internal_disconnect(id)
			}
			pub fn clear(&self) -> usize {
				self.inner.internal_clear()
			}
		}
	};
	(shared $vis:vis struct $name:ident >> $msg:ty) => {
		$vis struct $name<Sender: ?Sized>
		where $msg: Send
		{
			inner: $crate::core::signal::VxSignal<Sender, $msg, $crate::core::signal::VxSharedSignalPolicy>,
		}

		impl<Sender: ?Sized> Clone for $name<Sender> {
			fn clone(&self) -> Self {
				Self { inner: self.inner.clone() }
			}
		}

		impl<Sender: ?Sized> $name<Sender> {
			pub fn new() -> Self {
				Self {
					inner: $crate::core::signal::VxSignal::new(),
				}
			}
			#[inline(always)]
			pub fn emit(&self, sender: &mut Sender, msg: &$msg) {
				self.inner.emit(sender, &msg);
			}
			pub fn connect<F>(
				&self, f: F
			) -> $crate::types::gen_vector::VxGenIndex
			where F: Fn(&mut Sender, &$msg) + Send + Sync + 'static
			{
				self.inner.internal_connect(Box::new(f))
			}
			pub fn disconnect(&self, id: $crate::types::gen_vector::VxGenIndex) -> bool {
				self.inner.internal_disconnect(id)
			}
			pub fn clear(&self) -> usize {
				self.inner.internal_clear()
			}
		}
	};
}