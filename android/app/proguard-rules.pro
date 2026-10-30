# JNA and the UniFFI bindings are reached through reflection / native callbacks.
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keep class dev.rubi.core.** { *; }
-dontwarn java.awt.**
