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
    airplay_context_reset, airplay_context_state, airplay_listen_bound_addr,
    airplay_listen_start, airplay_listen_state, airplay_listen_stop,
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

/// Start listening on an explicit local interface. `addr` is a UTF-8 IP literal
/// with port (e.g. "127.0.0.1:7000"). `authorized` must be 1 AND the caller (Kotlin)
/// must already have obtained explicit user consent — the core re-checks it.
/// Returns the bound port (>0) on success, or a negative error code:
/// -1 bad args, -3 invalid/wildcard address, -8 not authorized, -9 already
/// listening, -10 bind/IO failure, -127 native panic.
#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeListenStart(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    addr: JByteArray,
    authorized: jint,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        let bytes = match env.convert_byte_array(&addr) {
            Ok(b) => b,
            Err(_) => return -1,
        };
        if bytes.is_empty() || bytes.len() > 128 {
            return -1;
        }
        airplay_listen_start(
            handle as *mut std::ffi::c_void,
            bytes.as_ptr(),
            bytes.len(),
            authorized,
        ) as jint
    }))
    .unwrap_or(-127)
}

/// Stop listening. Returns 0 on success, -1 if not listening / bad handle.
#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeListenStop(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        airplay_listen_stop(handle as *mut std::ffi::c_void) as jint
    }))
    .unwrap_or(-127)
}

/// 0 = not listening, >0 = bound port, -1 = bad handle.
#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeListenState(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        airplay_listen_state(handle as *mut std::ffi::c_void) as jint
    }))
    .unwrap_or(-127)
}

/// Read the actual bound address string into `out`. Returns byte count, or the
/// required capacity if `out` is too small (caller grows and retries).
#[no_mangle]
pub extern "system" fn Java_com_example_airplaytv_ffi_AirPlayProtocol_nativeListenBoundAddr(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    out: JByteArray,
    out_cap: jint,
) -> jint {
    catch_unwind(AssertUnwindSafe(|| {
        let cap = out_cap as usize;
        if cap == 0 {
            return -1;
        }
        let mut buf = vec![0u8; cap];
        let res = unsafe {
            airplay_listen_bound_addr(
                handle as *mut std::ffi::c_void,
                buf.as_mut_ptr(),
                buf.len(),
            )
        };
        if res >= 0 {
            let n = (res as usize).min(cap);
            if n > 0 {
                let _ = env.set_byte_array_region(&out, 0, &buf[..n]);
            }
        }
        res as jint
    }))
    .unwrap_or(-127)
}
