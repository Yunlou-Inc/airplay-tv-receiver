package com.example.airplaytv.media

enum class AdapterState { NEW, CONFIGURED, STARTED, STOPPED, RELEASED }
abstract class LifecycleAdapter {
    var state = AdapterState.NEW; private set
    protected fun transition(expected: Set<AdapterState>, next: AdapterState, action: () -> Unit = {}) {
        check(state in expected) { "invalid transition $state -> $next" }; action(); state = next
    }
    fun start() = transition(setOf(AdapterState.CONFIGURED), AdapterState.STARTED, ::onStart)
    fun stop() = transition(setOf(AdapterState.STARTED), AdapterState.STOPPED, ::onStop)
    fun flush() { check(state == AdapterState.STARTED); onFlush() }
    fun release() { if (state != AdapterState.RELEASED) { onRelease(); state = AdapterState.RELEASED } }
    protected open fun onStart() {}; protected open fun onStop() {}; protected open fun onFlush() {}; protected open fun onRelease() {}
}
/** Platform boundary only: no AirPlay decryption or actual mirroring is implemented. */
abstract class MediaCodecAdapter : LifecycleAdapter() {
    fun configure(format: VideoFormat) = transition(setOf(AdapterState.NEW, AdapterState.STOPPED), AdapterState.CONFIGURED) { onConfigure(format) }
    abstract fun queue(unit: AccessUnit, timeoutUs: Long): Boolean
    protected abstract fun onConfigure(format: VideoFormat)
}
abstract class AudioTrackAdapter : LifecycleAdapter() {
    fun configure(format: AudioFormat) = transition(setOf(AdapterState.NEW, AdapterState.STOPPED), AdapterState.CONFIGURED) { onConfigure(format) }
    abstract fun write(unit: AccessUnit, timeoutMs: Long): Int
    protected abstract fun onConfigure(format: AudioFormat)
}
