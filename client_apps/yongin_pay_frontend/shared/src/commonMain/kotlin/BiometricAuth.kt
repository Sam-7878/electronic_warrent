package com.myapplication

/**
 * Multiplatform Biometric Authentication wrapper.
 * In Android, this maps to BiometricPrompt (Fingerprint/FaceID).
 * In Desktop, this is typically bypassed or uses a PIN simulation.
 */
expect fun authenticateWithBiometrics(
    reason: String,
    onSuccess: () -> Unit,
    onError: (String) -> Unit
)
