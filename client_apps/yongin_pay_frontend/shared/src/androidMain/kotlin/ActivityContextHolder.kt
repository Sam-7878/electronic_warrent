package com.myapplication

import androidx.fragment.app.FragmentActivity

/**
 * Holds the current FragmentActivity so that KMP shared code can access it for BiometricPrompt.
 */
object ActivityContextHolder {
    var currentActivity: FragmentActivity? = null
}
