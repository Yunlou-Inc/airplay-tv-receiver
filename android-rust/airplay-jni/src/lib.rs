//! JNI bridge for Android TV.
//!
//! Thin adapter that exposes the `airplay-protocol` C-ABI to Kotlin via JNI.
//! The Kotlin counterpart is `com.example.airplaytv.ffi.AirPlayProtocol`.
//!
//! BUILD: produced by `cargo-ndk` for each target ABI (arm64-v8a, armeabi-v7a, x86_64),
//! output `libairplay_jni.so`, placed under `app/src/main/jniLibs/<abi>/`.
//!
//! STATUS: authored in a sandbox without the Android NDK or network access.
//! NOT compiled or verified. The `jni` crate call shapes follow `jni` 0.21; review
//! against the pinned version (lifetime on `JNIEnv`, exact method names) before building.
//! See `docs/ADR-001-ffi-integration.md`.

use jni::JNIEnv;
use jni::objects::{JByteArray, JClass};
use jni::sys::{jint, jlong};
use std::panic::{catch_unwind, AssertUnwindSafe};

// Re-exported from the dependency-free core crate (ffi module is `pub`).
use airplay_protocol::ffi::{
    airplay_context_create, airplay_context_destroy, airplay_context_handle,
    airplay_context_reset, airplay_context_state,
};

const MAX_FFI_INPUT: usize = 64 * 1024;

/// Create a context. Returns 0 on failure (matches the core's null-handle contract).
#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeCreate(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    catch_unwind(AssertUnwindSafe(|| airplay_context_create() as jlong)).unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        airplay_context_destroy(handle as *mut std::ffi::c_void);
    }));
}

#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeReset(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        airplay_context_reset(handle as *mut std::ffi::c_void) as jint
    }))
    .unwrap_or(-127)
}

#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeState(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        airplay_context_state(handle as *mut std::ffi::c_void) as jint
    }))
    .unwrap_or(-127)
}

#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeHandle(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    input: JByteArray,
    input_len: jint,
    output: JByteArray,
    output_cap: jint,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        let input_bytes = match env.convert_byte_array(&input) {
            Ok(b) => b,
            Err(_) => return -1,
        };
        if input_bytes.len() > MAX_FFI_INPUT || input_len < 0 {
            return -1;
        }
        let cap = output_cap as usize;
        if cap == 0 {
            return -1;
        }
        let mut out_buf = vec![0u8; cap];
        // `airplay_context_handle` is the only `unsafe` export in the core FFI.
        let res = unsafe {
            airplay_context_handle(
                handle as *mut std::ffi::c_void,
                input_bytes.as_ptr(),
                input_bytes.len(),
                out_buf.as_mut_ptr(),
                out_buf.len(),
            )
        };
        if res >= 0 {
            let n = (res as usize).min(cap);
            if n > 0 {
                let _ = env.set_byte_array_region(&output, 0, &out_buf[..n]);
            }
        }
        res as jint
    }))
    .unwrap_or(-127)
}
