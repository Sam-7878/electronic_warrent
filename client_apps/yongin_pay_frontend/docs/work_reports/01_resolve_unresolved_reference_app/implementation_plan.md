# Fix Gradle JVM Compatibility and Missing Declarations

This plan addresses the build failure caused by Gradle JVM version incompatibility and the missing `expect` declaration for `getPlatformName`.

## Proposed Changes

### Build Configuration

#### [gradle-wrapper.properties](file:///D:/_Work/MC_and_nGNN_for_GoG/yongin_pay_frontend/gradle/wrapper/gradle-wrapper.properties)

- Upgrade Gradle version from 8.2.1 to 8.7 to support Java 21.

```diff
-distributionUrl=https\://services.gradle.org/distributions/gradle-8.2.1-bin.zip
+distributionUrl=https\://services.gradle.org/distributions/gradle-8.7-bin.zip
```

---

### Shared Module - Common Sources

#### [NEW] [Platform.kt](file:///D:/_Work/MC_and_nGNN_for_GoG/yongin_pay_frontend/shared/src/commonMain/kotlin/Platform.kt)

- Add the missing `expect` declaration for `getPlatformName()`.

```kotlin
// Root package to match actual implementations in main.*.kt
expect fun getPlatformName(): String
```

#### [App.kt](file:///D:/_Work/MC_and_nGNN_for_GoG/yongin_pay_frontend/shared/src/commonMain/kotlin/App.kt)

- Add missing import for `authenticateWithBiometrics`.

```diff
 // Root package

+import com.myapplication.authenticateWithBiometrics
 import androidx.compose.foundation.background
```

---

### Shared Module - iOS Sources

#### [NEW] [BiometricAuth.ios.kt](file:///D:/_Work/MC_and_nGNN_for_GoG/yongin_pay_frontend/shared/src/iosMain/kotlin/BiometricAuth.ios.kt)

- Add a dummy implementation for iOS to prevent build failure (since the `expect` declaration exists but no `actual` was found for iOS).

```kotlin
package com.myapplication

actual fun authenticateWithBiometrics(
    reason: String,
    onSuccess: () -> Unit,
    onError: (String) -> Unit
) {
    // Simulator/Demo implementation
    onSuccess()
}
```

## Verification Plan

### Automated Tests
- Run `:shared:assembleDebug` to verify the build succeeds.
- Run `./gradlew build` to ensure all targets (Android, Desktop, iOS) are compatible.

### Manual Verification
- Check if the Gradle sync in Android Studio completes without errors.
