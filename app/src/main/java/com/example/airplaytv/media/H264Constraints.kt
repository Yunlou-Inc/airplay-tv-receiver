package com.example.airplaytv.media

data class H264Limits(val maxNalBytes: Int = 2 * 1024 * 1024, val maxWidth: Int = 1920,
                      val maxHeight: Int = 1080, val maxFrameRate: Int = 30,
                      val maxParameterSetBytes: Int = 4 * 1024)
class H264Validator(private val limits: H264Limits = H264Limits()) {
    fun validateNalSize(size: Int) = require(size in 1..limits.maxNalBytes) { "NAL size out of bounds" }
    fun validate(format: VideoFormat) {
        require(format.codec == VideoCodec.H264)
        require(format.width <= limits.maxWidth && format.height <= limits.maxHeight)
        require(format.frameRate <= limits.maxFrameRate)
        require(format.sps.size in 1..limits.maxParameterSetBytes && format.pps.size in 1..limits.maxParameterSetBytes)
        require((format.sps[0].toInt() and 0x1f) == 7) { "invalid SPS NAL type" }
        require((format.pps[0].toInt() and 0x1f) == 8) { "invalid PPS NAL type" }
    }
}
