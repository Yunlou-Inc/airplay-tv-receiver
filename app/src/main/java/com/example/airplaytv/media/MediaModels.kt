package com.example.airplaytv.media

import java.nio.ByteBuffer

data class MediaTimestamp(val presentationTimeUs: Long) { init { require(presentationTimeUs >= 0) } }

enum class VideoCodec { H264 }
enum class AudioCodec { AAC_LC, PCM_16BIT }
data class VideoFormat(val codec: VideoCodec, val width: Int, val height: Int, val frameRate: Int,
                       val sps: ByteArray, val pps: ByteArray) {
    init { require(width in 16..3840); require(height in 16..2160); require(frameRate in 1..60); require(sps.isNotEmpty()); require(pps.isNotEmpty()) }
}
data class AudioFormat(val codec: AudioCodec, val sampleRateHz: Int, val channelCount: Int) {
    init { require(sampleRateHz in 8_000..192_000); require(channelCount in 1..8) }
}
interface SharedBuffer { val size: Int; fun readOnlyBuffer(): ByteBuffer; fun close() }
class ByteArraySharedBuffer(bytes: ByteArray) : SharedBuffer {
    private var data: ByteArray? = bytes.copyOf()
    override val size get() = data?.size ?: 0
    override fun readOnlyBuffer(): ByteBuffer = ByteBuffer.wrap(checkNotNull(data) { "buffer closed" }).asReadOnlyBuffer()
    override fun close() { data?.fill(0); data = null }
}
data class AccessUnit(val track: Track, val timestamp: MediaTimestamp, val data: SharedBuffer,
                      val keyFrame: Boolean = false, val endOfStream: Boolean = false) { enum class Track { VIDEO, AUDIO } }
