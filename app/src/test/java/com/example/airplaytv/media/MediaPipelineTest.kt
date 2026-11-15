package com.example.airplaytv.media

import org.junit.Assert.*
import org.junit.Test

class MediaPipelineTest {
    private fun au(n: Int) = AccessUnit(AccessUnit.Track.VIDEO, MediaTimestamp(1), ByteArraySharedBuffer(ByteArray(n)))
    @Test fun queueBoundsBackpressureFlushAndClose() {
        val q = BoundedMediaQueue(2, 5)
        assertEquals(OfferResult.ACCEPTED, q.offer(au(3)))
        assertEquals(OfferResult.FULL, q.offer(au(3)))
        assertEquals(1 to 3, q.snapshot()); q.flush(); assertEquals(0 to 0, q.snapshot())
        assertNull(q.poll(1)); q.close(); assertEquals(OfferResult.CLOSED, q.offer(au(1)))
    }
    @Test fun h264ConstraintsRejectOversizeAndWrongParameterTypes() {
        val v = H264Validator(H264Limits(maxNalBytes = 4))
        v.validateNalSize(4)
        assertThrows(IllegalArgumentException::class.java) { v.validateNalSize(5) }
        assertThrows(IllegalArgumentException::class.java) {
            v.validate(VideoFormat(VideoCodec.H264, 1920, 1080, 30, byteArrayOf(1), byteArrayOf(8)))
        }
    }
    @Test fun lifecycleIsStrictAndReleaseIdempotent() {
        val adapter = object : MediaCodecAdapter() {
            override fun onConfigure(format: VideoFormat) {}
            override fun queue(unit: AccessUnit, timeoutUs: Long) = state == AdapterState.STARTED
        }
        assertThrows(IllegalStateException::class.java) { adapter.start() }
        adapter.configure(VideoFormat(VideoCodec.H264, 1280, 720, 30, byteArrayOf(7), byteArrayOf(8)))
        adapter.start(); adapter.flush(); adapter.stop(); adapter.release(); adapter.release()
        assertEquals(AdapterState.RELEASED, adapter.state)
    }
}
