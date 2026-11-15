package com.example.airplaytv.discovery

/** Conservative DNS-SD configuration. Ports belong to future listeners; advertising alone opens none. */
data class DiscoveryConfig(
    val enabled: Boolean = false,
    val instanceName: String = "AirPlay TV Receiver",
    val airPlayPort: Int = 7000,
    val raopPort: Int = 5000,
    val deviceId: String,
    val model: String = "AndroidTV",
    val sourceVersion: String = "0.1",
    val features: String = "0x0",
) {
    init {
        require(instanceName.isNotBlank() && instanceName.length <= 63)
        require(airPlayPort in 1..65535 && raopPort in 1..65535)
        require(DEVICE_ID.matches(deviceId))
        require(SAFE_VALUE.matches(model) && SAFE_VALUE.matches(sourceVersion) && SAFE_VALUE.matches(features))
    }

    companion object {
        private val DEVICE_ID = Regex("^[0-9A-Fa-f]{2}(:[0-9A-Fa-f]{2}){5}$")
        private val SAFE_VALUE = Regex("^[ -~]{1,128}$")
    }
}

data class ServiceRecord(val type: String, val name: String, val port: Int, val attributes: Map<String, String>)

object AirPlayRecords {
    fun from(config: DiscoveryConfig): List<ServiceRecord> = listOf(
        ServiceRecord(
            type = "_airplay._tcp.", name = config.instanceName, port = config.airPlayPort,
            attributes = linkedMapOf(
                "deviceid" to config.deviceId.uppercase(), "features" to config.features,
                "model" to config.model, "srcvers" to config.sourceVersion,
            ),
        ),
        ServiceRecord(
            type = "_raop._tcp.", name = "${config.deviceId.replace(":", "").uppercase()}@${config.instanceName}",
            port = config.raopPort,
            attributes = linkedMapOf(
                "ch" to "2", "cn" to "0", "et" to "0", "md" to "0", "pw" to "false",
                "sr" to "44100", "ss" to "16", "tp" to "TCP,UDP", "vn" to "3",
            ),
        ),
    )
}

sealed interface DiscoveryEvent {
    data object Starting : DiscoveryEvent
    data class Published(val serviceTypes: Set<String>) : DiscoveryEvent
    data object Stopped : DiscoveryEvent
    data object NetworkUnavailable : DiscoveryEvent
    data object NetworkChanged : DiscoveryEvent
    data class NameConflict(val serviceType: String) : DiscoveryEvent
    data class Error(val operation: String, val code: Int) : DiscoveryEvent
}

fun interface DiscoveryEventSink { fun onEvent(event: DiscoveryEvent) }

interface DiscoveryController {
    fun start(config: DiscoveryConfig)
    fun stop()
}
