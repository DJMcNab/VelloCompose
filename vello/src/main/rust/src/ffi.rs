use std::sync::{Arc, Mutex};

use jni::{
    objects::{JClass, JObject},
    sys::{jint, jlong},
    JNIEnv,
};
use ndk::native_window::NativeWindow;

use crate::{
    util::{abort_on_panic, INIT},
    VelloState,
};

/// Trick the linker into keeping this library
#[unsafe(no_mangle)]
pub extern "C" fn linker_trick_rust() {}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_linebender_vello_Vello_initRust<'local>(
    _: JNIEnv<'local>,
    _: JClass<'local>,
) {
    let _ = &*INIT;
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_linebender_vello_Vello_initialise<'local>(
    _: JNIEnv<'local>,
    _: JObject<'local>,
) -> jlong {
    abort_on_panic(|| {
        let state = VelloState::new();
        let state = Arc::new(Mutex::new(state));

        let state = Arc::into_raw(state) as usize;
        bytemuck::cast(state)
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_linebender_vello_Vello_setColor<'local>(
    _: JNIEnv<'local>,
    _: JObject<'local>,
    state: jlong,
    color: jint,
) {
    abort_on_panic(|| {
        // Safety: We only call this function with a value created from `initialise`.
        let state = unsafe { access_stored_state(state) };
        let mut state = state.lock().unwrap();
        state.color = color;
        state.render_default();
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_linebender_vello_Vello_setSurface<'local>(
    env: JNIEnv<'local>,
    _: JObject<'local>,
    state: jlong,
    surface: JObject<'local>,
) {
    abort_on_panic(|| unsafe {
        let state = access_stored_state(state);
        let window = NativeWindow::from_surface(env.get_native_interface(), *surface).unwrap();
        let mut state = state.lock().unwrap();
        state.set_window(window);
        state.render_default();
    })
}

unsafe fn access_stored_state(state: jlong) -> Arc<Mutex<VelloState>> {
    let value: usize = bytemuck::cast(state);
    let ptr = value as *const Mutex<VelloState>;
    assert!(!ptr.is_null());
    unsafe {
        Arc::increment_strong_count(ptr);
        Arc::from_raw(ptr)
    }
}
