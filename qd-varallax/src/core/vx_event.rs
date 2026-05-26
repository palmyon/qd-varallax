use winit::window::WindowId;

use crate::{abstractions::{abstract_windows::VxWindowBuilder}, types::geometry::{VxSize, VxVec2}};


pub type VxKey = winit::keyboard::KeyCode;
pub type VxMouseButton = winit::event::MouseButton;

pub struct VxMouseEvent {
	/// イベント発生場所(絶対座標)
	pos: VxVec2,
	/// マウスボタン
	button: Option<VxMouseButton>,
	/// マウスホイールの回転軸と回転量
	wheel_delta: VxVec2,
}

impl VxMouseEvent {
	#[inline]
	pub(crate) fn new(pos: VxVec2, button: Option<VxMouseButton>, wheel_delta: VxVec2) -> Self {
		Self {
			pos,
			button,
			wheel_delta,
		}
	}
	#[inline]
	pub fn pos(&self) -> VxVec2 { self.pos }
	#[inline]
	pub fn button(&self) -> Option<VxMouseButton> { self.button }
	#[inline]
	pub fn wheel_delta(&self) -> VxVec2 { self.wheel_delta }
}

pub struct VxKeyEvent {
	key: VxKey,
	/// 押されたならTrue, 離れたならFalse
	is_pressed: bool,
}

impl VxKeyEvent {
	#[inline]
	pub(crate) fn new(key: VxKey, is_pressed: bool) -> Self {
		Self {
			key,
			is_pressed,
		}
	}
	#[inline]
	pub fn key(&self) -> VxKey { self.key }
	#[inline]
	pub fn is_pressed(&self) -> bool { self.is_pressed }
}

pub struct VxWindowEvent {
	size: VxSize,
}

impl VxWindowEvent {
	#[inline]
	pub(crate) fn new(size: VxSize) -> Self {
		Self { size }
	}
	#[inline]
	pub fn size(&self) -> VxSize { self.size }
}

pub enum VxEvent {
	//マウスイベント
	MousePressEvent { event: VxMouseEvent },
	MouseReleaseEvent { event: VxMouseEvent },
	MouseMoveEvent { event: VxMouseEvent },
	MouseWheelEvent { event: VxMouseEvent },
	//キーボードイベント
	KeyPressedEvent { event: VxKeyEvent },
	KeyReleasedEvent { event: VxKeyEvent },
	//ウィンドウイベント
	ResizeEvent { event: VxWindowEvent },
	ShowEvent { builder: Box<dyn VxWindowBuilder> },
	CloseEvent { window_id: WindowId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VxEventResult {
	/// 処理を終了
	Accept,
	/// 処理を無視
	Ignore,
}