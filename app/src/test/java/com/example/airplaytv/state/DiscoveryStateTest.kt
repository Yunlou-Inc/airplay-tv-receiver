package com.example.airplaytv.state

import com.example.airplaytv.discovery.DiscoveryEvent
import org.junit.Assert.*
import org.junit.Test

class DiscoveryStateTest {
    @Test fun eventsReachApplicationState() {
        val starting = reduce(ReceiverUiState(), ReceiverAction.DiscoveryChanged(DiscoveryEvent.Starting))
        assertEquals(DiscoveryPhase.STARTING, starting.discovery.phase)
        val published = reduce(starting, ReceiverAction.DiscoveryChanged(DiscoveryEvent.Published(setOf("_airplay._tcp.", "_raop._tcp."))))
        assertEquals(DiscoveryPhase.PUBLISHED, published.discovery.phase)
        val failed = reduce(published, ReceiverAction.DiscoveryChanged(DiscoveryEvent.Error("register", 3)))
        assertEquals(DiscoveryPhase.ERROR, failed.discovery.phase)
        assertEquals("register:3", failed.discovery.message)
    }
}
