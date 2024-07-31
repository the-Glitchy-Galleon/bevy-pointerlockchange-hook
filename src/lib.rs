//! This Plugin emits an event whenever the pointer lock changes through the browser.
//!
//! This is necessary, because the browser eats the input (Escape key) that
//! would cause the cursor to unlock.
//! That's pretty unhelpful if the browser forgets to make the cursor visible again,
//! and no amount of goodwill on the bevy side is gonna fix unsynced behaviour without enabling this hook.
use bevy::prelude::*;

pub struct PointerLockChangePlugin;

#[derive(Event)]
pub struct PointerLockChangedByBrowser {
	pub is_pointer_locked: bool,
}

impl Plugin for PointerLockChangePlugin {
	#[cfg(not(target_family = "wasm"))]
	fn build(&self, app: &mut App) {
		app.add_event::<PointerLockChangedByBrowser>();
	}

	#[cfg(target_family = "wasm")]
	fn build(&self, app: &mut App) {
		app.add_event::<PointerLockChangedByBrowser>()
			.add_systems(Startup, wasm_stuff::setup_pointer_lock_listener)
			.add_systems(PostUpdate, wasm_stuff::write_event);
	}
}

#[cfg(target_family = "wasm")]
mod wasm_stuff {
	use super::*;
	use std::sync::atomic::{AtomicBool, Ordering};
	use wasm_bindgen::prelude::*;
	use web_sys::{js_sys::Function, window};

	static mut IS_ACTIVE: AtomicBool = AtomicBool::new(false);
	static mut WAS_ACTIVE: AtomicBool = AtomicBool::new(false);

	#[wasm_bindgen]
	extern "C" {
		#[wasm_bindgen(js_namespace = window, js_name = addEventListener)]
		fn add_event_listener(event_type: &str, handler: &Function);
	}

	#[wasm_bindgen]
	pub fn setup_pointer_lock_listener() {
		let window = window().unwrap();
		let document = window.document().unwrap();

		let closure = Closure::wrap(Box::new(move || {
			let is_active = document.pointer_lock_element().is_some();

			unsafe {
				WAS_ACTIVE.store(
					IS_ACTIVE.swap(is_active, Ordering::SeqCst),
					Ordering::SeqCst,
				);
			}
		}) as Box<dyn FnMut()>);

		window
			.add_event_listener_with_callback("pointerlockchange", closure.as_ref().unchecked_ref())
			.expect("Failed to add event listener");
		closure.forget();
	}

	pub fn write_event(mut events: EventWriter<PointerLockChangedByBrowser>) {
		let is_active = unsafe { IS_ACTIVE.load(Ordering::Relaxed) };
		let was_active = unsafe { WAS_ACTIVE.swap(is_active, Ordering::Relaxed) };

		if was_active != is_active {
			events.send(PointerLockChangedByBrowser {
				is_pointer_locked: is_active,
			});
		}
	}
}

// fn debug_pointer_lock_events(mut events: EventReader<PointerLockChangedByBrowserEvent>) {
// 	for event in events.read() {
// 		if event.is_pointer_locked {
// 			info!("Pointer lock is active");
// 		} else {
// 			info!("Pointer lock is not active");
// 		}
// 	}
// }

// #[cfg(target_family = "wasm")]
// fn setup_event_listener() {
// 	let handler = Closure::wrap(Box::new(move |event: JsValue| {
// 		let event: CustomEvent = event.into();
// 		let detail: String = event.detail().as_string().unwrap_or_default();
// 		info!("{}", format!("Received event with detail: {}", detail));
// 	}) as Box<dyn FnMut(JsValue)>);

// 	unsafe {
// 		add_event_listener("custom_event", handler.as_ref().unchecked_ref());
// 	}
// 	handler.forget(); // Prevent Rust from dropping the closure
// }

// #[cfg(target_family = "wasm")]
// fn propagate_esc(keys: Res<ButtonInput<KeyCode>>) {
// 	if keys.just_pressed(KeyCode::Escape) {
// 		trigger_event("escape", "pressed")
// 	}
// 	if keys.just_released(KeyCode::Escape) {
// 		trigger_event("escape", "released")
// 	}
// }

// #[wasm_bindgen]
// pub fn trigger_event(event_name: &str, detail: &str) {
// 	let window = window().expect("should have a window in this context");
// 	let document = window.document().expect("should have a document on window");

// 	let event = web_sys::CustomEvent::new_with_event_init_dict(
// 		event_name,
// 		web_sys::CustomEventInit::new().detail(&JsValue::from_str(detail)),
// 	)
// 	.unwrap();
// 	document.dispatch_event(&event).unwrap();
// }
