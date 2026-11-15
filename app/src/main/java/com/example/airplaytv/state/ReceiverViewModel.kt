package com.example.airplaytv.state

import androidx.compose.runtime.State
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.example.airplaytv.discovery.DiscoveryConfig
import com.example.airplaytv.discovery.DiscoveryController
import com.example.airplaytv.discovery.DiscoveryEvent
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class ReceiverViewModel : ViewModel() {
    private val mutableUiState = mutableStateOf(ReceiverUiState())
    val uiState: State<ReceiverUiState> = mutableUiState

    private var controller: DiscoveryController? = null
    private var stopJob: Job? = null
    private var requested = false

    fun attachDiscoveryController(value: DiscoveryController) {
        if (controller === value) return
        controller?.stop()
        controller = value
        // Deliberately do not start here: discovery is user initiated and defaults off.
    }

    fun dispatch(action: ReceiverAction) {
        mutableUiState.value = reduce(mutableUiState.value, action)
    }

    fun enableDiscovery(config: DiscoveryConfig, durationSeconds: Int = 300) {
        if (requested) return
        require(durationSeconds > 0)
        requested = true
        stopJob?.cancel()
        mutableUiState.value = mutableUiState.value.copy(
            discoveryEnabled = true,
            discoverySecondsRemaining = durationSeconds,
            discoveryPermissionDenied = false,
        )
        controller?.start(config.copy(enabled = true))
        stopJob = viewModelScope.launch {
            for (remaining in durationSeconds - 1 downTo 0) {
                delay(1_000)
                mutableUiState.value = mutableUiState.value.copy(discoverySecondsRemaining = remaining)
            }
            disableDiscovery()
        }
    }

    fun permissionDenied() {
        mutableUiState.value = mutableUiState.value.copy(discoveryPermissionDenied = true)
        disableDiscovery(preservePermissionDenied = true)
    }

    fun disableDiscovery(preservePermissionDenied: Boolean = false) {
        if (requested) controller?.stop()
        requested = false
        stopJob?.cancel()
        stopJob = null
        mutableUiState.value = mutableUiState.value.copy(
            discoveryEnabled = false,
            discoverySecondsRemaining = 0,
            discoveryPermissionDenied = if (preservePermissionDenied) true else false,
            discovery = DiscoveryStatus(),
        )
    }

    fun onDiscoveryEvent(event: DiscoveryEvent) = dispatch(ReceiverAction.DiscoveryChanged(event))

    override fun onCleared() {
        disableDiscovery()
        controller = null
    }
}
