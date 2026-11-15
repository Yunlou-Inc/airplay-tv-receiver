package com.example.airplaytv.discovery

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.content.ContextCompat

object DiscoveryPermission {
    const val REQUEST_CODE = 4101
    fun requiredPermission(): String? =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) Manifest.permission.NEARBY_WIFI_DEVICES else null

    fun isGranted(context: Context): Boolean = requiredPermission()?.let {
        ContextCompat.checkSelfPermission(context, it) == PackageManager.PERMISSION_GRANTED
    } ?: true
}
