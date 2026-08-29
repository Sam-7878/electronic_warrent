package com.goatbank.ngofund

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import io.ktor.client.*
import io.ktor.client.engine.cio.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.client.statement.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

// HETE Backend URL
const val BASE_URL = "http://127.0.0.1:8080/api/v1/ngo"

val httpClient = HttpClient(CIO) {
    install(ContentNegotiation) {
        json(Json {
            ignoreUnknownKeys = true
            prettyPrint = true
        })
    }
}

// Data Models
@Serializable
data class LogEntry(val id: Int, val timestamp: String, val message: String, val color: String = "White")

enum class FundState {
    INITIAL, DEPOSITED, VP_SUBMITTED, SETTLED, FROZEN
}

fun main() = application {
    val windowState = androidx.compose.ui.window.rememberWindowState(
        width = 1200.dp,
        height = 800.dp
    )
    Window(title = "NGO Fund Smart Contract - HETE Network", onCloseRequest = ::exitApplication, state = windowState) {
        MaterialTheme(
            colorScheme = darkColorScheme(
                background = Color(0xFF121212),
                surface = Color(0xFF1E1E1E),
                primary = Color(0xFF00FFCC), // Neon Cyan
                secondary = Color(0xFFB000FF), // Neon Purple
                error = Color(0xFFFF0055) // Neon Pink/Red
            )
        ) {
            Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
                NgoDashboard()
            }
        }
    }
}

@Composable
fun NgoDashboard() {
    val coroutineScope = rememberCoroutineScope()
    var fundState by remember { mutableStateOf(FundState.INITIAL) }
    val logs = remember { mutableStateListOf<LogEntry>() }
    var logCounter by remember { mutableStateOf(0) }
    
    fun addLog(msg: String, color: String = "White") {
        logs.add(0, LogEntry(logCounter++, java.time.LocalTime.now().toString(), msg, color))
    }

    Row(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        // Left Panel: Dashboard
        Column(modifier = Modifier.weight(1f).padding(end = 16.dp)) {
            Text("BURDE NGO Escrow Dashboard", style = MaterialTheme.typography.headlineMedium, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Bold)
            Spacer(modifier = Modifier.height(24.dp))
            
            // Escrow Status Card
            Card(
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
                modifier = Modifier.fillMaxWidth().border(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.3f), RoundedCornerShape(12.dp))
            ) {
                Column(modifier = Modifier.padding(24.dp)) {
                    Text("Escrow Pool Status", style = MaterialTheme.typography.titleMedium, color = Color.Gray)
                    Spacer(modifier = Modifier.height(16.dp))
                    
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                        Column {
                            Text("Fund ID", color = Color.Gray)
                            Text("1001", color = Color.White, fontWeight = FontWeight.Bold)
                        }
                        Column {
                            Text("Purpose Code", color = Color.Gray)
                            Text("WATER_INFRASTRUCTURE", color = MaterialTheme.colorScheme.secondary, fontWeight = FontWeight.Bold)
                        }
                        Column {
                            Text("Target Village", color = Color.Gray)
                            Text("did:goat:village-A", color = Color.White, fontWeight = FontWeight.Bold)
                        }
                    }
                    
                    Spacer(modifier = Modifier.height(24.dp))
                    
                    // Balance indicator
                    val balance = when(fundState) {
                        FundState.INITIAL -> "0.0"
                        FundState.SETTLED -> "0.0"
                        else -> "50,000.0"
                    }
                    val balanceColor = if (fundState == FundState.FROZEN) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
                    
                    Text("Locked Balance", color = Color.Gray)
                    Text("$balance HETE_STABLE", style = MaterialTheme.typography.displaySmall, color = balanceColor, fontWeight = FontWeight.Bold)
                    
                    Spacer(modifier = Modifier.height(16.dp))
                    // Status Badge
                    val statusText = when(fundState) {
                        FundState.INITIAL -> "AWAITING DEPOSIT"
                        FundState.DEPOSITED -> "LOCKED (READY FOR VP)"
                        FundState.VP_SUBMITTED -> "VP VERIFIED (AWAITING MULTI-SIG)"
                        FundState.SETTLED -> "ATOMIC SETTLEMENT COMPLETED"
                        FundState.FROZEN -> "FROZEN BY E-WARRANT"
                    }
                    Box(modifier = Modifier.clip(RoundedCornerShape(8.dp)).background(balanceColor.copy(alpha = 0.2f)).padding(horizontal = 16.dp, vertical = 8.dp)) {
                        Text(statusText, color = balanceColor, fontWeight = FontWeight.Bold)
                    }
                }
            }
            
            Spacer(modifier = Modifier.height(24.dp))
            Text("Blockchain Audit Trail", style = MaterialTheme.typography.titleMedium, color = Color.Gray)
            Spacer(modifier = Modifier.height(8.dp))
            
            // Audit Logs
            LazyColumn(modifier = Modifier.fillMaxSize().background(Color.Black).padding(8.dp)) {
                items(logs, key = { it.id }) { log ->
                    val textColor = when(log.color) {
                        "Cyan" -> MaterialTheme.colorScheme.primary
                        "Purple" -> MaterialTheme.colorScheme.secondary
                        "Red" -> MaterialTheme.colorScheme.error
                        "Green" -> Color(0xFF00FF00)
                        else -> Color.White
                    }
                    Text("[${log.timestamp}] ${log.message}", color = textColor, style = MaterialTheme.typography.bodySmall)
                    Divider(color = Color.DarkGray, thickness = 0.5.dp, modifier = Modifier.padding(vertical = 4.dp))
                }
            }
        }
        
        // Right Panel: Presenter Controls
        Column(modifier = Modifier.width(350.dp)) {
            Text("Presenter Controls", style = MaterialTheme.typography.titleLarge, color = Color.White)
            Spacer(modifier = Modifier.height(24.dp))
            
            // Step 1: Deposit
            ScenarioButton(
                title = "1. NGO Deposit",
                description = "Lock funds with Purpose Code",
                icon = Icons.Default.AccountBalance,
                color = MaterialTheme.colorScheme.primary,
                enabled = fundState == FundState.INITIAL,
                onClick = {
                    coroutineScope.launch {
                        try {
                            val payload = """
                                {
                                  "fund_id": "1001",
                                  "ngo_did": "did:goat:ngo-africa-01",
                                  "amount": 50000.0,
                                  "currency": "HETE_STABLE",
                                  "purpose_code": "WATER_INFRASTRUCTURE",
                                  "target_village": "did:goat:village-A"
                                }
                            """.trimIndent()
                            addLog("Sending Deposit Request...", "White")
                            val response = httpClient.post("$BASE_URL/deposit") {
                                contentType(ContentType.Application.Json)
                                setBody(payload)
                            }
                            if (response.status.isSuccess()) {
                                fundState = FundState.DEPOSITED
                                addLog("✓ Fund Locked in Escrow. TxHash generated.", "Cyan")
                            }
                        } catch (e: Exception) {
                            addLog("Error: ${e.message}", "Red")
                        }
                    }
                }
            )
            
            // Step 2: Vendor VP
            ScenarioButton(
                title = "2. Vendor VP Submit",
                description = "Cross-verify Vendor VC",
                icon = Icons.Default.VerifiedUser,
                color = MaterialTheme.colorScheme.secondary,
                enabled = fundState == FundState.DEPOSITED,
                onClick = {
                    coroutineScope.launch {
                        try {
                            val payload = """
                                {
                                  "fund_id": "1001",
                                  "vendor_did": "did:goat:vendor-water-99",
                                  "claim_amount": 50000.0,
                                  "vp_credential": {
                                    "type": "AuthorizedVendorCredential",
                                    "authorized_purpose": "WATER_INFRASTRUCTURE",
                                    "signature": "sig_vendor_99"
                                  }
                                }
                            """.trimIndent()
                            addLog("Vendor submitting VP...", "White")
                            val response = httpClient.post("$BASE_URL/claim") {
                                contentType(ContentType.Application.Json)
                                setBody(payload)
                            }
                            if (response.status.isSuccess()) {
                                fundState = FundState.VP_SUBMITTED
                                addLog("✓ VP Verified! Purpose Code Matched 100%.", "Purple")
                            }
                        } catch (e: Exception) {
                            addLog("Error: ${e.message}", "Red")
                        }
                    }
                }
            )
            
            // Step 3: Approve
            ScenarioButton(
                title = "3. Village Multi-sig",
                description = "Approve and Atomic Settlement",
                icon = Icons.Default.CheckCircle,
                color = Color(0xFF00FF00),
                enabled = fundState == FundState.VP_SUBMITTED,
                onClick = {
                    coroutineScope.launch {
                        try {
                            val payload = """
                                {
                                  "fund_id": "1001",
                                  "village_did": "did:goat:village-A",
                                  "multi_sig": "sig_village_A_multi"
                                }
                            """.trimIndent()
                            addLog("Village Leader signing transaction...", "White")
                            val response = httpClient.post("$BASE_URL/approve") {
                                contentType(ContentType.Application.Json)
                                setBody(payload)
                            }
                            if (response.status.isSuccess()) {
                                fundState = FundState.SETTLED
                                addLog("✓ Atomic Settlement Complete (T+0). Vendor paid.", "Green")
                            }
                        } catch (e: Exception) {
                            addLog("Error: ${e.message}", "Red")
                        }
                    }
                }
            )
            
            Divider(modifier = Modifier.padding(vertical = 16.dp), color = Color.DarkGray)
            
            // Step 4: Freeze (can happen if deposited or vp_submitted)
            ScenarioButton(
                title = "🚨 Emergency Freeze",
                description = "Execute RegTech E-Warrant",
                icon = Icons.Default.Warning,
                color = MaterialTheme.colorScheme.error,
                enabled = fundState == FundState.DEPOSITED || fundState == FundState.VP_SUBMITTED,
                onClick = {
                    coroutineScope.launch {
                        try {
                            val payload = """
                                {
                                  "fund_id": "1001",
                                  "warrant_vc": {
                                    "issuer": "did:goat:law-enforcement",
                                    "target_did": "did:goat:vendor-water-99",
                                    "reason": "FRAUD_DETECTED"
                                  }
                                }
                            """.trimIndent()
                            addLog("Receiving E-Warrant from Law Enforcement...", "Red")
                            val response = httpClient.post("$BASE_URL/freeze") {
                                contentType(ContentType.Application.Json)
                                setBody(payload)
                            }
                            if (response.status.isSuccess()) {
                                fundState = FundState.FROZEN
                                addLog("🚨 ASSETS FROZEN. Smart Contract Isolated target DID.", "Red")
                            }
                        } catch (e: Exception) {
                            addLog("Error: ${e.message}", "Red")
                        }
                    }
                }
            )
            
            Spacer(modifier = Modifier.weight(1f))
            Button(
                onClick = {
                    fundState = FundState.INITIAL
                    logs.clear()
                    addLog("System Reset", "White")
                },
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(containerColor = Color.DarkGray)
            ) {
                Text("Reset Demo")
            }
        }
    }
}

@Composable
fun ScenarioButton(title: String, description: String, icon: androidx.compose.ui.graphics.vector.ImageVector, color: Color, enabled: Boolean, onClick: () -> Unit) {
    Button(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier.fillMaxWidth().height(80.dp).padding(bottom = 12.dp),
        colors = ButtonDefaults.buttonColors(containerColor = color.copy(alpha = 0.2f), contentColor = color, disabledContainerColor = Color.DarkGray.copy(alpha = 0.2f)),
        shape = RoundedCornerShape(12.dp),
        border = if (enabled) androidx.compose.foundation.BorderStroke(1.dp, color) else null
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
            Icon(imageVector = icon, contentDescription = null, modifier = Modifier.size(32.dp))
            Spacer(modifier = Modifier.width(16.dp))
            Column {
                Text(title, fontWeight = FontWeight.Bold, style = MaterialTheme.typography.titleMedium)
                Text(description, style = MaterialTheme.typography.bodySmall, color = if (enabled) Color.LightGray else Color.Gray)
            }
        }
    }
}
