#![allow(
    unsafe_code,
    reason = "Higher-level deny is intended to be scoped in lib.rs module, but this is a submodule of that"
)]

use std::sync::{Arc, Mutex};

use jni::{
    objects::{JClass, JLongArray, JObject, JString},
    sys::{jfloat, jint, jlong},
    JNIEnv,
};
use ndk::native_window::NativeWindow;

use crate::{
    util::{abort_on_panic, INIT},
    SurfaceKind, VelloJni,
};

struct FfiState {
    vello: VelloJni,
    updated_surfaces_scratch: Vec<jlong>,
}

/// Trick the linker into keeping this library around
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
        let vello = VelloJni::new();
        let state = FfiState {
            vello,
            updated_surfaces_scratch: Vec::with_capacity(20),
        };
        let state = Arc::new(Mutex::new(state));

        let state = Arc::<Mutex<FfiState>>::into_raw(state) as usize;
        bytemuck::cast(state)
    })
}

/// # Safety
///
/// - `env` must be a valid JNI environment
/// - `surface` must be a `Surface` associated with `env`
/// - `state` must be a value which was returned from [`Java_org_linebender_vello_Vello_initialise`]
///    and which has not been freed.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_linebender_vello_Vello_newSurface<'local>(
    env: JNIEnv<'local>,
    _: JObject<'local>,
    state: jlong,
    surface: JObject<'local>,
    surface_id: jlong,
    width: jint,
    height: jint,
) {
    abort_on_panic(|| {
        // Safety: Precondition of this function that state is correct
        let state = unsafe { access_stored_state(state) };
        let mut state = state.lock().unwrap();
        assert!(!surface.is_null());

        // Safety: This is probably a valid surface.
        let window =
            unsafe { NativeWindow::from_surface(env.get_native_interface(), *surface).unwrap() };
        state.vello.new_window(
            window,
            surface_id,
            width.try_into().unwrap(),
            height.try_into().unwrap(),
        );
    })
}

/// # Safety
///
/// - `env` must be a valid JNI environment
/// - `state` must be a value which was returned from [`Java_org_linebender_vello_Vello_initialise`]
///    and which has not been freed.
/// - `updated_surfaces` must be a valid Long Array from Java.
///
/// # Aborts
///
/// If `updated_surfaces` does not contain at least `n_updated_surfaces`.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_linebender_vello_Vello_doRender<'local>(
    env: JNIEnv<'local>,
    _: JObject<'local>,
    state: jlong,
    updated_surfaces: JLongArray<'local>,
    n_updated_surfaces: jint,
) {
    abort_on_panic(|| {
        // Safety: Precondition of this function that state is correct
        let state = unsafe { access_stored_state(state) };
        let mut state = state.lock().unwrap();
        let state = &mut *state;

        let len = n_updated_surfaces.try_into().unwrap();
        state.updated_surfaces_scratch.resize(len, 0);
        env.get_long_array_region(&updated_surfaces, 0, &mut state.updated_surfaces_scratch)
            .unwrap();
        state.vello.perform_render(&state.updated_surfaces_scratch);
    });
}

/// # Safety
///
/// - `env` must be a valid JNI environment
/// - `state` must be a value which was returned from [`Java_org_linebender_vello_Vello_initialise`]
///    and which has not been freed.
/// - `text` must be a valid `String` from Java.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_linebender_vello_Vello_makeVariableFontSurface<'local>(
    mut env: JNIEnv<'local>,
    _: JObject<'local>,
    state: jlong,
    surface_id: jlong,
    text: JString<'local>,
    font_size: jfloat,
    font_weight: jfloat,
) {
    abort_on_panic(|| {
        // Safety: Precondition of this function that state is correct
        let state = unsafe { access_stored_state(state) };
        let mut state = state.lock().unwrap();
        let text = env.get_string(&text).unwrap().into();
        let surface = state
            .vello
            .surfaces
            .get_mut(&surface_id)
            .expect("Tried to make a variable font surface for an invalid surface");
        surface.kind = SurfaceKind::VariableFont {
            text,
            size: font_size,
            weight: font_weight,
        }
    })
}

/// # Safety
///
/// - `state` must be a value which was returned from [`Java_org_linebender_vello_Vello_initialise`]
///    and which has not been freed.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_linebender_vello_Vello_updateVariableFontParameters<
    'local,
>(
    _: JNIEnv<'local>,
    _: JObject<'local>,
    state: jlong,
    surface_id: jlong,
    font_size: jfloat,
    font_weight: jfloat,
) {
    abort_on_panic(|| {
        // Safety: Precondition of this function that state is correct
        let state = unsafe { access_stored_state(state) };
        let mut state = state.lock().unwrap();
        let surface = state
            .vello
            .surfaces
            .get_mut(&surface_id)
            .expect("Tried to make a variable font surface for an invalid surface");
        if let SurfaceKind::VariableFont { size, weight, .. } = &mut surface.kind {
            *size = font_size;
            *weight = font_weight;
        }
    })
}

/// # Safety
///
/// - `env` must be a valid JNI environment
/// - `state` must be a value which was returned from [`Java_org_linebender_vello_Vello_initialise`]
///    and which has not been freed.
/// - `new_text` must be a valid `String` from Java.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_linebender_vello_Vello_updateVariableFontText<'local>(
    mut env: JNIEnv<'local>,
    _: JObject<'local>,
    state: jlong,
    surface_id: jlong,
    new_text: JString<'local>,
) {
    abort_on_panic(|| {
        // Safety: Precondition of this function that state is correct
        let state = unsafe { access_stored_state(state) };
        let mut state = state.lock().unwrap();
        let new_text = env.get_string(&new_text).unwrap().into();
        let surface = state
            .vello
            .surfaces
            .get_mut(&surface_id)
            .expect("Tried to make a variable font surface for an invalid surface");
        if let SurfaceKind::VariableFont { text, .. } = &mut surface.kind {
            *text = new_text;
        }
    })
}

/// Access a stored
/// - `state` must be a value which was returned from [`Java_org_linebender_vello_Vello_initialise`]
///    and which has not been freed.
unsafe fn access_stored_state(state: jlong) -> Arc<Mutex<FfiState>> {
    let value: usize = bytemuck::cast(state);
    let ptr = value as *const Mutex<FfiState>;
    assert!(!ptr.is_null());
    unsafe {
        Arc::increment_strong_count(ptr);
        Arc::from_raw(ptr)
    }
}
