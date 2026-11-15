package com.example.airplaytv.state

data class PairedDevice(val id: String, val name: String)

enum class ReceiverPhase { WAITING, PAIRING, STREAMING }

data class ReceiverUiState(
    val phase: ReceiverPhase = ReceiverPhase.WAITING,
    val pairingCode: String? = null,
    val streamingDevice: String? = null,
    val pairedDevices: List<PairedDevice> = emptyList(),
    val requirePairing: Boolean = true,
    val discoveryEnabled: Boolean = false,
    val discoverySecondsRemaining: Int = 0,
    val discoveryPermissionDenied: Boolean = false,
    val discovery: DiscoveryStatus = DiscoveryStatus(),
)

sealed interface ReceiverAction {
    data class PairingRequested(val code: String) : ReceiverAction
    data class PairingConfirmed(val device: PairedDevice) : ReceiverAction
    data class StreamStarted(val deviceName: String) : ReceiverAction
    data object StreamStopped : ReceiverAction
    data class PairingRequirementChanged(val required: Boolean) : ReceiverAction
    data class DiscoveryChanged(val event: com.example.airplaytv.discovery.DiscoveryEvent) : ReceiverAction
}

fun reduce(state: ReceiverUiState, action: ReceiverAction): ReceiverUiState = when (action) {
    is ReceiverAction.PairingRequested -> state.copy(phase = ReceiverPhase.PAIRING, pairingCode = action.code)
    is ReceiverAction.PairingConfirmed -> state.copy(
        phase = ReceiverPhase.WAITING,
        pairingCode = null,
        pairedDevices = (state.pairedDevices + action.device).distinctBy(PairedDevice::id),
    )
    is ReceiverAction.StreamStarted -> state.copy(phase = ReceiverPhase.STREAMING, streamingDevice = action.deviceName)
    ReceiverAction.StreamStopped -> state.copy(phase = ReceiverPhase.WAITING, streamingDevice = null)
    is ReceiverAction.PairingRequirementChanged -> state.copy(requirePairing = action.required)
    is ReceiverAction.DiscoveryChanged -> state.copy(discovery = state.discovery.reduce(action.event))
}

enum class DiscoveryPhase { OFF, STARTING, PUBLISHED, UNAVAILABLE, ERROR }

data class DiscoveryStatus(
    val phase: DiscoveryPhase = DiscoveryPhase.OFF,
    val serviceTypes: Set<String> = emptySet(),
    val message: String? = null,
)

fun DiscoveryStatus.reduce(event: com.example.airplaytv.discovery.DiscoveryEvent): DiscoveryStatus = when (event) {
    com.example.airplaytv.discovery.DiscoveryEvent.Starting -> copy(phase = DiscoveryPhase.STARTING, message = null)
    is com.example.airplaytv.discovery.DiscoveryEvent.Published -> copy(phase = DiscoveryPhase.PUBLISHED, serviceTypes = event.serviceTypes, message = null)
    com.example.airplaytv.discovery.DiscoveryEvent.Stopped -> DiscoveryStatus()
    com.example.airplaytv.discovery.DiscoveryEvent.NetworkUnavailable -> copy(phase = DiscoveryPhase.UNAVAILABLE, serviceTypes = emptySet(), message = "network-unavailable")
    com.example.airplaytv.discovery.DiscoveryEvent.NetworkChanged -> copy(phase = DiscoveryPhase.STARTING, serviceTypes = emptySet(), message = "network-changed")
    is com.example.airplaytv.discovery.DiscoveryEvent.NameConflict -> copy(message = "name-conflict:${event.serviceType}")
    is com.example.airplaytv.discovery.DiscoveryEvent.Error -> copy(phase = DiscoveryPhase.ERROR, serviceTypes = emptySet(), message = "${event.operation}:${event.code}")
}
