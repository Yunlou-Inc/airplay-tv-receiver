package com.example.airplaytv.ffi

import androidx.annotation.Keep

/**
 * JNI facade over the Rust `airplay-protocol` C-ABI.
 * Design contract: [docs/ADR-001-ffi-integration.md].
 *
 * STATUS: authored in a sandbox — NOT built or verified. The native library
 * `airplay_jni` (produced by `android-rust/airplay-jni` via cargo-ndk and placed in
 * `jniLibs/<abi>/libairplay_jni.so`) is loaded on first access. Until that build exists,
 * [isAvailable] is false and every call short-circuits to [ProtocolResult.Unavailable]
 * without touching native code. This file is additive and compiles independently of the
 * native library.
 *
 * Contract mirrors the Rust FFI exactly:
 *  - handles are opaque ids (`Long` on this side, never the raw pointer);
 *  - create returns 0 on failure;
 *  - handle() returns positive = bytes written; if value > outputCap the caller must grow
 *    the buffer and retry; negatives are errors (-1 bad args/null/overflow, -2 parse,
 *    -3 lock, -4 route, -127 panic caught with no unwind).
 */
object AirPlayProtocol {

    val isAvailable: Boolean = try {
        System.loadLibrary("airplay_jni")
        true
    } catch (_: UnsatisfiedLinkError) {
        false
    } catch (_: SecurityException) {
        false
    }

    @Keep
    private external fun nativeCreate(): Long

    @Keep
    private external fun nativeDestroy(handle: Long)

    @Keep
    private external fun nativeReset(handle: Long): Int

    @Keep
    private external fun nativeState(handle: Long): Int

    @Keep
    private external fun nativeHandle(
        handle: Long,
        input: ByteArray,
        inputLen: Int,
        output: ByteArray,
        outputCap: Int
    ): Int

    /** Creates a protocol context. Returns null on failure. */
    fun createContext(): Long? {
        if (!isAvailable) return null
        val h = nativeCreate()
        return if (h == 0L) null else h
    }

    fun destroyContext(handle: Long) {
        if (isAvailable) nativeDestroy(handle)
    }

    fun reset(handle: Long): ProtocolResult<Unit> {
        if (!isAvailable) return ProtocolResult.Unavailable
        return mapCode(nativeReset(handle))
    }

    fun state(handle: Long): ProtocolResult<SessionState> {
        if (!isAvailable) return ProtocolResult.Unavailable
        val code = nativeState(handle)
        if (code < 0) return mapCode(code)
        return ProtocolResult.Ok(SessionState.fromCode(code))
    }

    /**
     * Feeds a raw RTSP request to the context and returns the response bytes.
     * Retries automatically when the native side reports the output buffer was too small.
     */
    fun handle(handle: Long, request: ByteArray): ProtocolResult<ByteArray> {
        if (!isAvailable) return ProtocolResult.Unavailable
        if (request.size > MAX_FFI_INPUT) return ProtocolResult.Err(-1, "input exceeds 64 KiB")
        var outCap = (request.size + 256).coerceAtLeast(256)
        repeat(4) {
            val out = ByteArray(outCap)
            val res = nativeHandle(handle, request, request.size, out, outCap)
            when {
                res < 0 -> return mapCode(res)
                res <= outCap -> return ProtocolResult.Ok(out.copyOf(res))
                else -> outCap = res + 16 // native returned required size; grow and retry
            }
        }
        return ProtocolResult.Err(-1, "output buffer growth exhausted")
    }

    private fun mapCode(code: Int): ProtocolResult<Nothing> = when (code) {
        -1 -> ProtocolResult.Err(-1, "invalid args / null handle / input overflow")
        -2 -> ProtocolResult.Err(-2, "parse error")
        -3 -> ProtocolResult.Err(-3, "lock / state error")
        -4 -> ProtocolResult.Err(-4, "route error")
        -127 -> ProtocolResult.Err(-127, "native panic (caught, no unwind)")
        else -> ProtocolResult.Err(code, "unknown code")
    }

    const val MAX_FFI_INPUT: Int = 64 * 1024
}

sealed class ProtocolResult<out T> {
    data class Ok<T>(val value: T) : ProtocolResult<T>()
    data class Err(val code: Int, val message: String? = null) : ProtocolResult<Nothing>()
    data object Unavailable : ProtocolResult<Nothing>()
}

enum class SessionState(val code: Int) {
    IDLE(0),
    ANNOUNCED(1),
    CONFIGURED(2),
    STREAMING(3),
    PAUSED(4),
    CLOSED(5);

    companion object {
        fun fromCode(c: Int): SessionState = entries.firstOrNull { it.code == c } ?: IDLE
    }
}
