package com.example.airplaytv.service

import android.app.Service
import android.content.Context
import android.content.Intent
import android.net.ConnectivityManager
import android.net.nsd.NsdManager
import android.os.Binder
import android.os.IBinder
import com.example.airplaytv.discovery.DiscoveryConfig
import com.example.airplaytv.discovery.DiscoveryController
import com.example.airplaytv.discovery.DiscoveryEvent
import com.example.airplaytv.discovery.NsdDiscoveryController

/** Private service boundary. Discovery is disabled until an in-process client explicitly enables it. */
class ReceiverService : Service() {
    private lateinit var discovery: DiscoveryController
    private val binder = LocalBinder()

    override fun onCreate() {
        super.onCreate()
        discovery = NsdDiscoveryController(
            getSystemService(Context.NSD_SERVICE) as NsdManager,
            getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager,
        ) { binder.emit(it) }
    }

    override fun onBind(intent: Intent?): IBinder = binder
    override fun onDestroy() { discovery.stop(); super.onDestroy() }

    inner class LocalBinder : Binder() {
        fun interface Listener { fun onEvent(event: DiscoveryEvent) }
        private var listener: Listener? = null
        fun setListener(value: Listener?) { listener = value }
        fun startDiscovery(config: DiscoveryConfig) = discovery.start(config)
        fun stopDiscovery() = discovery.stop()
        internal fun emit(event: DiscoveryEvent) = listener?.onEvent(event)
    }
}
