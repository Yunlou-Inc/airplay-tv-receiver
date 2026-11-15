package com.example.airplaytv

import android.content.Context
import android.net.ConnectivityManager
import android.net.nsd.NsdManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.airplaytv.discovery.NsdDiscoveryController
import com.example.airplaytv.state.ReceiverViewModel
import com.example.airplaytv.ui.ReceiverApp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            val vm: ReceiverViewModel = viewModel()
            val controller = remember(vm) {
                NsdDiscoveryController(
                    getSystemService(Context.NSD_SERVICE) as NsdManager,
                    getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager,
                    vm::onDiscoveryEvent,
                )
            }
            DisposableEffect(controller) {
                vm.attachDiscoveryController(controller)
                onDispose { vm.disableDiscovery() }
            }
            ReceiverApp(vm)
        }
    }
}
