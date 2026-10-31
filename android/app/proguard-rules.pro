# JNA and the UniFFI bindings are reached through reflection / native callbacks.
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keep class dev.rubi.core.** { *; }
-dontwarn java.awt.**

# The Geetest page calls back into the app through @JavascriptInterface methods by name.
-keepclassmembers class * {
    @android.webkit.JavascriptInterface <methods>;
}
-keepattributes JavascriptInterface
