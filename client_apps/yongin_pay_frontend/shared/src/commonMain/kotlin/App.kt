// Root package

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import com.myapplication.authenticateWithBiometrics
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay

data class Transaction(val id: String, val title: String, val amount: String, val date: String, val isPositive: Boolean)

@OptIn(ExperimentalAnimationApi::class)
@Composable
fun UserMobileApp() {
    var langToggle by remember { mutableStateOf(Strings.isKorean) }
    var isAuthenticating by remember { mutableStateOf(false) }
    var txSuccess by remember { mutableStateOf(false) }
    
    val initialHistory = listOf(
        Transaction("3", "GS25 (Convenience Store)", "-₩4,500", "Today, 14:30", false),
        Transaction("2", "Stablecoin Mint", "+₩50,000", "Yesterday", true),
        Transaction("1", "Netflix Subscription", "-₩17,000", "Oct 12", false)
    )
    val txHistoryList = remember { mutableStateListOf(*initialHistory.toTypedArray()) }
    var currentBalance by remember { mutableStateOf(1250000) }

    // Pulse Burst Animation State
    val pulseAnim = remember { Animatable(0f) }
    LaunchedEffect(txSuccess) {
        if (txSuccess) {
            pulseAnim.animateTo(
                targetValue = 1f,
                animationSpec = tween(durationMillis = 800, easing = FastOutSlowInEasing)
            )
        }
    }

    AppTheme {
        Box(modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
            // Pulse Canvas
            if (txSuccess) {
                val primaryColor = MaterialTheme.colorScheme.tertiary
                Canvas(modifier = Modifier.fillMaxSize()) {
                    val radius = pulseAnim.value * size.maxDimension
                    val alpha = 1f - pulseAnim.value
                    drawCircle(
                        color = primaryColor.copy(alpha = alpha * 0.5f),
                        radius = radius,
                        style = Stroke(width = 8.dp.toPx() * alpha)
                    )
                    drawCircle(
                        color = primaryColor.copy(alpha = alpha * 0.2f),
                        radius = radius * 0.8f
                    )
                }
            }

            Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
                // Header
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    Column {
                        Text(text = Strings.welcome, color = MaterialTheme.colorScheme.onSurface, fontSize = 14.sp)
                        Text(text = "Alex K.", color = MaterialTheme.colorScheme.onBackground, fontSize = 28.sp, fontWeight = FontWeight.Bold)
                    }
                    Button(
                        onClick = { Strings.toggleLanguage(); langToggle = Strings.isKorean },
                        colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.surface)
                    ) {
                        Text(if (langToggle) "EN" else "KO", color = MaterialTheme.colorScheme.primary)
                    }
                }

                Spacer(modifier = Modifier.height(32.dp))

                // Wallet Card with Holographic Sheen
                Box(modifier = Modifier.fillMaxWidth().height(220.dp).metalGradientBackground().holographicSheen().padding(24.dp)) {
                    Column(verticalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxSize()) {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                            Text("NOVA", color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Black, fontSize = 20.sp)
                            Box(modifier = Modifier.size(40.dp, 24.dp).background(Color(0xFFE0E0E0), RoundedCornerShape(4.dp))) 
                        }
                        
                        Column {
                            Text("Total Balance", color = Color.Gray, fontSize = 14.sp)
                            // Animated Ticker for Balance
                            AnimatedContent(
                                targetState = currentBalance,
                                transitionSpec = {
                                    if (targetState < initialState) {
                                        slideInVertically { height -> -height } + fadeIn() with slideOutVertically { height -> height } + fadeOut()
                                    } else {
                                        slideInVertically { height -> height } + fadeIn() with slideOutVertically { height -> -height } + fadeOut()
                                    }.using(SizeTransform(clip = false))
                                }
                            ) { balance ->
                                Text("₩%,d".format(balance), color = Color.White, fontSize = 40.sp, fontWeight = FontWeight.ExtraBold)
                            }
                        }
                    }
                }

                Spacer(modifier = Modifier.height(24.dp))

                // Animated Send Action Button
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        val buttonScale by animateFloatAsState(if (isAuthenticating) 1.1f else if (txSuccess) 1.2f else 1.0f)
                        
                        Button(
                            onClick = {
                                if (isAuthenticating || txSuccess) return@Button
                                isAuthenticating = true
                                authenticateWithBiometrics(
                                    reason = "Sign transaction with VP",
                                    onSuccess = {
                                        isAuthenticating = false
                                        txSuccess = true
                                        currentBalance -= 10000
                                        txHistoryList.add(0, Transaction(System.currentTimeMillis().toString(), "Sent to Friend", "-₩10,000", "Just now", false))
                                    },
                                    onError = { isAuthenticating = false }
                                )
                            },
                            modifier = Modifier
                                .size(80.dp)
                                .scale(buttonScale)
                                .then(
                                    if (txSuccess) Modifier.neonGlow(MaterialTheme.colorScheme.tertiary, 40)
                                    else Modifier.animatedNeonBorder(MaterialTheme.colorScheme.primary, 40)
                                ),
                            shape = CircleShape,
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.surface)
                        ) {
                            AnimatedContent(targetState = isAuthenticating to txSuccess) { (auth, success) ->
                                when {
                                    success -> Text("✓", fontSize = 32.sp, color = MaterialTheme.colorScheme.tertiary)
                                    auth -> CircularProgressIndicator(color = MaterialTheme.colorScheme.primary, modifier = Modifier.size(24.dp))
                                    else -> Text("➤", fontSize = 24.sp, color = MaterialTheme.colorScheme.primary)
                                }
                            }
                        }
                        Spacer(modifier = Modifier.height(12.dp))
                        Text(Strings.send, color = MaterialTheme.colorScheme.onBackground, fontWeight = FontWeight.Bold)
                    }
                }

                Spacer(modifier = Modifier.height(32.dp))

                // Transaction History
                Text(Strings.txHistory, fontSize = 18.sp, color = MaterialTheme.colorScheme.onBackground, fontWeight = FontWeight.Bold)
                Spacer(modifier = Modifier.height(16.dp))
                
                LazyColumn(modifier = Modifier.fillMaxWidth().weight(1f), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    items(txHistoryList, key = { it.id }) { tx ->
                        // Animated Visibility for new items
                        AnimatedVisibility(
                            visible = true,
                            enter = slideInVertically(initialOffsetY = { -it }) + fadeIn(),
                        ) {
                            Row(modifier = Modifier.fillMaxWidth().glassmorphism().padding(16.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                                Column {
                                    Text(tx.title, color = MaterialTheme.colorScheme.onBackground, fontWeight = FontWeight.Medium)
                                    Text(tx.date, color = MaterialTheme.colorScheme.onSurface, fontSize = 12.sp)
                                }
                                Text(tx.amount, color = if (tx.isPositive) MaterialTheme.colorScheme.tertiary else Color.White, fontWeight = FontWeight.Bold)
                            }
                        }
                    }
                }
            }
        }
    }
}

// ... Kiosk and HQ Dashboard (Omitted from this edit for brevity, but I will include them to make sure file is complete)

@Composable
fun DesktopApp() {
    var role by remember { mutableStateOf("hq") }
    var langToggle by remember { mutableStateOf(Strings.isKorean) }
    
    AppTheme {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
            Column(modifier = Modifier.fillMaxSize().padding(24.dp)) {
                // Header
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text(Strings.appName, fontSize = 24.sp, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Black)
                    Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        Button(
                            onClick = { role = if (role == "hq") "kiosk" else "hq" },
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.surface)
                        ) {
                            Text(if (role == "hq") "Switch to Kiosk" else "Switch to HQ", color = Color.White)
                        }
                        Button(
                            onClick = { Strings.toggleLanguage(); langToggle = Strings.isKorean },
                            colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.surface)
                        ) {
                            Text(if (langToggle) "EN" else "KO", color = MaterialTheme.colorScheme.primary)
                        }
                    }
                }
                
                Spacer(modifier = Modifier.height(32.dp))
                
                if (role == "hq") {
                    HQDashboard()
                } else {
                    KioskDashboard()
                }
            }
        }
    }
}

@Composable
fun HQDashboard() {
    var tps by remember { mutableStateOf(2841) }
    var nodes by remember { mutableStateOf(1248) }
    
    LaunchedEffect(Unit) {
        while (true) {
            delay(1000)
            tps += (-50..50).random()
            if (Math.random() > 0.9) nodes += (-1..2).random()
        }
    }

    Row(modifier = Modifier.fillMaxSize(), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
        Column(modifier = Modifier.weight(1f).fillMaxHeight().glassmorphism().padding(24.dp)) {
            Text(Strings.hqMode, color = MaterialTheme.colorScheme.primary, fontSize = 20.sp, fontWeight = FontWeight.Bold)
            Spacer(modifier = Modifier.height(32.dp))
            Button(
                onClick = {}, 
                colors = ButtonDefaults.buttonColors(containerColor = Color.Red.copy(alpha = 0.1f)), 
                modifier = Modifier.fillMaxWidth().animatedNeonBorder(Color.Red, 16)
            ) {
                Text(Strings.fdsAlerts, color = Color.Red, modifier = Modifier.padding(8.dp))
            }
            Spacer(modifier = Modifier.height(16.dp))
            Button(
                onClick = {}, 
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.secondary.copy(alpha = 0.1f)), 
                modifier = Modifier.fillMaxWidth().animatedNeonBorder(MaterialTheme.colorScheme.secondary, 16)
            ) {
                Text(Strings.executeWarrant, color = MaterialTheme.colorScheme.secondary, modifier = Modifier.padding(8.dp))
            }
        }
        
        Column(modifier = Modifier.weight(3f).fillMaxHeight()) {
            Text(Strings.networkHealth, fontSize = 24.sp, color = MaterialTheme.colorScheme.onBackground, fontWeight = FontWeight.Bold)
            Spacer(modifier = Modifier.height(24.dp))
            
            Box(modifier = Modifier.fillMaxWidth().weight(1f).metalGradientBackground().holographicSheen().padding(24.dp), contentAlignment = Alignment.Center) {
                Box(modifier = Modifier.fillMaxSize().border(1.dp, Color.White.copy(alpha=0.05f))) {
                    Text("GLOBAL DID NETWORK MAP", color = MaterialTheme.colorScheme.primary.copy(alpha=0.5f), fontSize = 32.sp, fontWeight = FontWeight.Black, modifier = Modifier.align(Alignment.Center))
                }
            }
            
            Spacer(modifier = Modifier.height(24.dp))
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Box(modifier = Modifier.weight(1f).glassmorphism().padding(24.dp)) {
                    Column {
                        Text(Strings.activeNodes, color = MaterialTheme.colorScheme.onSurface)
                        Text("$nodes", color = MaterialTheme.colorScheme.primary, fontSize = 36.sp, fontWeight = FontWeight.Black)
                    }
                }
                Box(modifier = Modifier.weight(1f).glassmorphism().padding(24.dp)) {
                    Column {
                        Text(Strings.tps, color = MaterialTheme.colorScheme.onSurface)
                        Text("$tps", color = MaterialTheme.colorScheme.secondary, fontSize = 36.sp, fontWeight = FontWeight.Black)
                    }
                }
            }
        }
    }
}

@Composable
fun KioskDashboard() {
    var step by remember { mutableStateOf("HOME") } 
    var inputAmount by remember { mutableStateOf("") }
    
    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally, 
            modifier = Modifier.width(600.dp).glassmorphism(32).padding(48.dp)
        ) {
            AnimatedContent(targetState = step) { targetStep ->
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    when (targetStep) {
                        "HOME" -> {
                            Text(Strings.kioskMode, fontSize = 36.sp, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Black)
                            Text("Convenience Store Terminal", color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.padding(top = 8.dp, bottom = 48.dp))
                            
                            Row(horizontalArrangement = Arrangement.spacedBy(32.dp)) {
                                Box(
                                    modifier = Modifier.size(220.dp)
                                        .background(MaterialTheme.colorScheme.surface, RoundedCornerShape(24.dp))
                                        .animatedNeonBorder(MaterialTheme.colorScheme.primary, 24)
                                        .clickable { step = "AMOUNT" }, 
                                    contentAlignment = Alignment.Center
                                ) {
                                    Text(Strings.mintStablecoin, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 18.sp)
                                }
                                Box(
                                    modifier = Modifier.size(220.dp)
                                        .background(MaterialTheme.colorScheme.surface, RoundedCornerShape(24.dp))
                                        .animatedNeonBorder(MaterialTheme.colorScheme.secondary, 24)
                                        .clickable { },
                                    contentAlignment = Alignment.Center
                                ) {
                                    Text(Strings.burnStablecoin, color = Color.White, fontWeight = FontWeight.Bold, fontSize = 18.sp)
                                }
                            }
                        }
                        "AMOUNT" -> {
                            Text(Strings.enterAmount, fontSize = 28.sp, color = Color.White, fontWeight = FontWeight.Bold)
                            Spacer(modifier = Modifier.height(32.dp))
                            Text(
                                text = if(inputAmount.isEmpty()) "₩0" else "₩%,d".format(inputAmount.toLong()), 
                                fontSize = 48.sp, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Black
                            )
                            Spacer(modifier = Modifier.height(32.dp))
                            val pads = listOf(listOf("1","2","3"), listOf("4","5","6"), listOf("7","8","9"), listOf("C","0","OK"))
                            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                                for (row in pads) {
                                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                        for (key in row) {
                                            Button(
                                                onClick = {
                                                    when(key) {
                                                        "C" -> inputAmount = ""
                                                        "OK" -> if(inputAmount.isNotEmpty()) step = "QR"
                                                        else -> inputAmount += key
                                                    }
                                                },
                                                modifier = Modifier.size(80.dp),
                                                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.surface),
                                                shape = RoundedCornerShape(16.dp)
                                            ) {
                                                Text(key, fontSize = 24.sp, color = Color.White)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "QR" -> {
                            Text(Strings.scanQr, fontSize = 24.sp, color = Color.White, fontWeight = FontWeight.Bold)
                            Spacer(modifier = Modifier.height(48.dp))
                            Box(modifier = Modifier.size(250.dp).animatedNeonBorder(MaterialTheme.colorScheme.secondary, 24).background(Color.Black)) {
                                var scanLineY by remember { mutableStateOf(0.dp) }
                                val scanAnim by animateDpAsState(
                                    targetValue = scanLineY,
                                    animationSpec = tween(1000, easing = LinearEasing)
                                )
                                LaunchedEffect(Unit) {
                                    scanLineY = 250.dp
                                    delay(1000)
                                    scanLineY = 0.dp
                                    delay(1000)
                                    step = "SUCCESS"
                                }
                                Box(modifier = Modifier.fillMaxWidth().height(4.dp).offset(y = scanAnim).background(MaterialTheme.colorScheme.primary))
                            }
                        }
                        "SUCCESS" -> {
                            Text("✓", fontSize = 80.sp, color = MaterialTheme.colorScheme.tertiary)
                            Spacer(modifier = Modifier.height(24.dp))
                            Text(Strings.mintComplete, fontSize = 28.sp, color = Color.White, fontWeight = FontWeight.Bold)
                            Spacer(modifier = Modifier.height(16.dp))
                            Text("₩%,d".format(if(inputAmount.isEmpty()) 0 else inputAmount.toLong()), fontSize = 40.sp, color = MaterialTheme.colorScheme.primary, fontWeight = FontWeight.Black)
                            Spacer(modifier = Modifier.height(48.dp))
                            Button(
                                onClick = { step = "HOME"; inputAmount = "" },
                                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary),
                                modifier = Modifier.fillMaxWidth().height(56.dp)
                            ) {
                                Text(Strings.backToHome, color = Color.Black, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                            }
                        }
                    }
                }
            }
        }
    }
}