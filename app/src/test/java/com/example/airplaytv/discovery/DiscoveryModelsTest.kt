package com.example.airplaytv.discovery

import org.junit.Assert.*
import org.junit.Test

class DiscoveryModelsTest {
    private val config = DiscoveryConfig(enabled = true, deviceId = "02:00:00:00:00:01")

    @Test fun createsOnlyRequiredConservativeRecords() {
        val records = AirPlayRecords.from(config)
        assertEquals(setOf("_airplay._tcp.", "_raop._tcp."), records.map { it.type }.toSet())
        assertEquals("0x0", records.first { it.type == "_airplay._tcp." }.attributes["features"])
        assertEquals("false", records.first { it.type == "_raop._tcp." }.attributes["pw"])
        assertFalse(records.any { it.attributes.keys.any { key -> key.contains("key", ignoreCase = true) } })
    }

    @Test(expected = IllegalArgumentException::class)
    fun rejectsInvalidDeviceId() { DiscoveryConfig(deviceId = "device") }

    @Test fun recordsUseConfiguredPortsAndCapabilities() {
        val records = AirPlayRecords.from(config.copy(airPlayPort = 7100, raopPort = 5100, features = "0x1"))
        assertEquals(7100, records[0].port)
        assertEquals(5100, records[1].port)
        assertEquals("0x1", records[0].attributes["features"])
    }
}
