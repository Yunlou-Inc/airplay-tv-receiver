package com.example.airplaytv.discovery

import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo

/** Registers only on a validated local-capable network and re-registers after network changes. */
class NsdDiscoveryController(
    private val nsd: NsdManager,
    private val connectivity: ConnectivityManager,
    private val sink: DiscoveryEventSink,
) : DiscoveryController {
    private var config: DiscoveryConfig? = null
    private val registrations = mutableMapOf<String, NsdManager.RegistrationListener>()
    private var callbackRegistered = false

    private val networkCallback = object : ConnectivityManager.NetworkCallback() {
        override fun onAvailable(network: Network) { restartForNetworkChange() }
        override fun onLost(network: Network) {
            if (!hasUsableNetwork()) { unregisterAll(); sink.onEvent(DiscoveryEvent.NetworkUnavailable) }
            else restartForNetworkChange()
        }
        override fun onCapabilitiesChanged(network: Network, caps: NetworkCapabilities) = restartForNetworkChange()
    }

    @Synchronized override fun start(config: DiscoveryConfig) {
        stop()
        this.config = config
        if (!config.enabled) { sink.onEvent(DiscoveryEvent.Stopped); return }
        sink.onEvent(DiscoveryEvent.Starting)
        runCatching { connectivity.registerDefaultNetworkCallback(networkCallback); callbackRegistered = true }
            .onFailure { sink.onEvent(DiscoveryEvent.Error("network-callback", -1)) }
        publishIfPossible()
    }

    @Synchronized override fun stop() {
        unregisterAll()
        if (callbackRegistered) runCatching { connectivity.unregisterNetworkCallback(networkCallback) }
        callbackRegistered = false
        config = null
        sink.onEvent(DiscoveryEvent.Stopped)
    }

    @Synchronized private fun restartForNetworkChange() {
        val current = config ?: return
        unregisterAll()
        sink.onEvent(DiscoveryEvent.NetworkChanged)
        if (current.enabled) publishIfPossible()
    }

    private fun hasUsableNetwork(): Boolean {
        val network = connectivity.activeNetwork ?: return false
        val caps = connectivity.getNetworkCapabilities(network) ?: return false
        return caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) &&
            !caps.hasCapability(NetworkCapabilities.NET_CAPABILITY_CAPTIVE_PORTAL)
    }

    @Synchronized private fun publishIfPossible() {
        val current = config ?: return
        if (!hasUsableNetwork()) { sink.onEvent(DiscoveryEvent.NetworkUnavailable); return }
        val records = AirPlayRecords.from(current)
        records.forEach { register(it) }
    }

    private fun register(record: ServiceRecord) {
        val info = NsdServiceInfo().apply {
            serviceName = record.name
            serviceType = record.type
            port = record.port
            record.attributes.forEach { (key, value) -> setAttribute(key, value) }
        }
        val requestedName = record.name
        val listener = object : NsdManager.RegistrationListener {
            override fun onServiceRegistered(actual: NsdServiceInfo) {
                if (actual.serviceName != requestedName) sink.onEvent(DiscoveryEvent.NameConflict(record.type))
                if (registrations.size == 2) sink.onEvent(DiscoveryEvent.Published(registrations.keys.toSet()))
            }
            override fun onRegistrationFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                registrations.remove(record.type)
                sink.onEvent(DiscoveryEvent.Error("register:${record.type}", errorCode))
            }
            override fun onServiceUnregistered(serviceInfo: NsdServiceInfo) = Unit
            override fun onUnregistrationFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
                sink.onEvent(DiscoveryEvent.Error("unregister:${record.type}", errorCode))
            }
        }
        registrations[record.type] = listener
        runCatching { nsd.registerService(info, NsdManager.PROTOCOL_DNS_SD, listener) }
            .onFailure { registrations.remove(record.type); sink.onEvent(DiscoveryEvent.Error("register:${record.type}", -1)) }
    }

    private fun unregisterAll() {
        registrations.values.toList().forEach { listener -> runCatching { nsd.unregisterService(listener) } }
        registrations.clear()
    }
}
