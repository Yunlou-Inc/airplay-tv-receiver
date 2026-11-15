package com.example.airplaytv.state

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class DiscoveryDefaultsTest {
    @Test
    fun discoveryIsOffByDefault() {
        val state = ReceiverUiState()
        assertFalse(state.discoveryEnabled)
        assertEquals(0, state.discoverySecondsRemaining)
        assertEquals(DiscoveryPhase.OFF, state.discovery.phase)
    }
}
