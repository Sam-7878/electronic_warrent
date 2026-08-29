package com.myapplication

actual fun authenticateWithBiometrics(
    reason: String,
    onSuccess: () -> Unit,
    onError: (String) -> Unit
) {
    // Simulator/Demo implementation
    onSuccess()
}
