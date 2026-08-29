package com.myapplication

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/**
 * Desktop implementation for Biometric Authentication.
 * Since Desktop typically lacks standardized biometric hardware without specific drivers,
 * we auto-approve or show a PIN simulation.
 */
actual fun authenticateWithBiometrics(
    reason: String,
    onSuccess: () -> Unit,
    onError: (String) -> Unit
) {
    // Simulate approval for desktop testing
    CoroutineScope(Dispatchers.Main).launch {
        delay(500)
        onSuccess()
    }
}
