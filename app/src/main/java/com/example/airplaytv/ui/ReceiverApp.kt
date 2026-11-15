package com.example.airplaytv.ui

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import com.example.airplaytv.discovery.DiscoveryConfig
import com.example.airplaytv.state.*

private const val DISCOVERY_DURATION_SECONDS = 300

@Composable
fun ReceiverApp(vm: ReceiverViewModel) {
    val state by vm.uiState
    val context = LocalContext.current
    val config = remember {
        val androidId = Settings.Secure.getString(context.contentResolver, Settings.Secure.ANDROID_ID).orEmpty()
        val hex = androidId.filter { it.isDigit() || it.lowercaseChar() in 'a'..'f' }.padEnd(12, '0').take(12)
        DiscoveryConfig(
            deviceId = hex.chunked(2).joinToString(":").ifEmpty { "02:00:00:00:00:00" },
        )
    }
    val permissionLauncher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        if (granted) vm.enableDiscovery(config, DISCOVERY_DURATION_SECONDS) else vm.permissionDenied()
    }
    val initialFocus = remember { FocusRequester() }

    MaterialTheme(colorScheme = darkColorScheme()) {
        Column(Modifier.fillMaxSize().background(Color(0xFF101418)).padding(40.dp)) {
            Text("AirPlay TV Receiver", fontSize = 30.sp)
            Spacer(Modifier.height(24.dp))
            StatusPanel(state)
            Spacer(Modifier.height(24.dp))
            Button(
                onClick = { vm.dispatch(ReceiverAction.PairingRequested("4821")) },
                modifier = Modifier.focusRequester(initialFocus),
            ) { Text("显示配对确认") }
            Spacer(Modifier.height(16.dp))
            if (state.phase == ReceiverPhase.PAIRING) {
                Text("请确认配对码：${state.pairingCode}")
                Button(onClick = { vm.dispatch(ReceiverAction.PairingConfirmed(PairedDevice("demo", "客厅设备"))) }) { Text("确认配对") }
            }
            Spacer(Modifier.height(20.dp))
            Text("已配对设备", fontSize = 20.sp)
            LazyColumn(Modifier.heightIn(max = 140.dp)) {
                if (state.pairedDevices.isEmpty()) item { Text("暂无设备") }
                items(state.pairedDevices, key = { it.id }) { Text("• ${it.name}", Modifier.padding(8.dp)) }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("安全设置：始终要求配对")
                Switch(state.requirePairing, { vm.dispatch(ReceiverAction.PairingRequirementChanged(it)) })
            }
            Spacer(Modifier.height(16.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("允许局域网设备发现（5 分钟后自动关闭）")
                Switch(
                    checked = state.discoveryEnabled,
                    onCheckedChange = { enabled ->
                        if (!enabled) vm.disableDiscovery()
                        else if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
                            ContextCompat.checkSelfPermission(context, Manifest.permission.NEARBY_WIFI_DEVICES) == PackageManager.PERMISSION_GRANTED
                        ) vm.enableDiscovery(config, DISCOVERY_DURATION_SECONDS)
                        else permissionLauncher.launch(Manifest.permission.NEARBY_WIFI_DEVICES)
                    },
                )
            }
            Text(discoveryText(state))
            if (state.discoveryEnabled) Text("剩余 ${state.discoverySecondsRemaining} 秒")
            if (state.discoveryPermissionDenied) Text("权限被拒绝，发现服务未启动。", color = MaterialTheme.colorScheme.error)
        }
        LaunchedEffect(Unit) { initialFocus.requestFocus() }
    }
}

private fun discoveryText(state: ReceiverUiState): String = when (state.discovery.phase) {
    DiscoveryPhase.OFF -> "发现状态：已关闭"
    DiscoveryPhase.STARTING -> "发现状态：正在启动"
    DiscoveryPhase.PUBLISHED -> "发现状态：已发布 ${state.discovery.serviceTypes.size} 项服务"
    DiscoveryPhase.UNAVAILABLE -> "发现状态：网络不可用"
    DiscoveryPhase.ERROR -> "发现状态：错误 ${state.discovery.message.orEmpty()}"
}

@Composable private fun StatusPanel(state: ReceiverUiState) {
    val text = when (state.phase) {
        ReceiverPhase.WAITING -> "等待投屏连接"
        ReceiverPhase.PAIRING -> "等待配对确认"
        ReceiverPhase.STREAMING -> "正在投屏：${state.streamingDevice.orEmpty()}"
    }
    Card { Text(text, Modifier.padding(24.dp), fontSize = 24.sp) }
}
