# Resolution Report: Unresolved reference: App

## Issue Description
Building the project with `./gradlew :shared:compileKotlinDesktop` failed with the following error:
`e: file:///D:/_Work/MC_and_nGNN_for_GoG/yongin_pay_frontend/shared/src/desktopMain/kotlin/main.desktop.kt:11:5 Unresolved reference: App`

The `@Preview` function `AppPreview` in `main.desktop.kt` was attempting to call a composable function named `App()`, which was not defined in the project. The project instead uses `MainView()`, `UserMobileApp()`, and `DesktopApp()`.

## Work Done

### 1. Fix `main.desktop.kt`
Updated the `AppPreview` function to call `MainView()` instead of the non-existent `App()`. `MainView()` is already configured to display `DesktopApp()`.

**File:** `shared/src/desktopMain/kotlin/main.desktop.kt`
```kotlin
@Preview
@Composable
fun AppPreview() {
    MainView()
}
```

### 2. Consistency update for `main.ios.kt`
The iOS implementation also had an unresolved reference to `App()`. I updated it to define a platform-specific `MainView()` and used it in the `MainViewController`.

**File:** `shared/src/iosMain/kotlin/main.ios.kt`
```kotlin
@Composable fun MainView() = UserMobileApp()

fun MainViewController() = ComposeUIViewController { MainView() }
```

## Results
- The "Unresolved reference: App" error is resolved.
- Platform-specific entry points are now consistent across Android, Desktop, and iOS.
- Desktop previews are now functional using `MainView()`.
