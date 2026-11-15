package com.example.airplaytv.state

import org.junit.Assert.*
import org.junit.Test

class ReceiverStateTest {
    @Test fun pairingFlowAddsDeviceAndReturnsToWaiting() {
        val pairing = reduce(ReceiverUiState(), ReceiverAction.PairingRequested("1234"))
        assertEquals(ReceiverPhase.PAIRING, pairing.phase)
        assertEquals("1234", pairing.pairingCode)
        val paired = reduce(pairing, ReceiverAction.PairingConfirmed(PairedDevice("1", "Phone")))
        assertEquals(ReceiverPhase.WAITING, paired.phase)
        assertEquals(listOf(PairedDevice("1", "Phone")), paired.pairedDevices)
        assertNull(paired.pairingCode)
    }

    @Test fun streamingFlowTracksAndClearsSender() {
        val streaming = reduce(ReceiverUiState(), ReceiverAction.StreamStarted("Mac"))
        assertEquals(ReceiverPhase.STREAMING, streaming.phase)
        assertEquals("Mac", streaming.streamingDevice)
        assertEquals(ReceiverUiState(), reduce(streaming, ReceiverAction.StreamStopped))
    }

    @Test fun pairingDeviceIsDeduplicatedById() {
        val first = reduce(ReceiverUiState(), ReceiverAction.PairingConfirmed(PairedDevice("1", "Old")))
        val second = reduce(first, ReceiverAction.PairingConfirmed(PairedDevice("1", "New")))
        assertEquals(1, second.pairedDevices.size)
    }
}
