package com.myapplication

import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat

/**
 * Android implementation for Biometric Authentication.
 * Uses the official androidx.biometric library to invoke the hardware prompt.
 */
actual fun authenticateWithBiometrics(
    reason: String,
    onSuccess: () -> Unit,
    onError: (String) -> Unit
) {
    val activity = ActivityContextHolder.currentActivity
    if (activity == null) {
        onError("Activity context is not initialized.")
        return
    }

    val executor = ContextCompat.getMainExecutor(activity)
    
    val biometricPrompt = BiometricPrompt(activity, executor,
        object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                super.onAuthenticationError(errorCode, errString)
                onError(errString.toString())
            }

            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                super.onAuthenticationSucceeded(result)
                onSuccess()
            }

            override fun onAuthenticationFailed() {
                super.onAuthenticationFailed()
                // Do not call onError immediately on failure, as the prompt allows multiple retries
                // until it triggers onAuthenticationError (e.g. too many attempts).
            }
        })

    val promptInfo = BiometricPrompt.PromptInfo.Builder()
        .setTitle(if (Strings.isKorean) "지문 인증" else "Biometric Authentication")
        .setSubtitle(reason)
        .setNegativeButtonText(if (Strings.isKorean) "취소" else "Cancel")
        .build()

    biometricPrompt.authenticate(promptInfo)
}
