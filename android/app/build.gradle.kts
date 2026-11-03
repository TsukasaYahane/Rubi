plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

// The Rust workspace sits one level above the Gradle project.
val rustRoot = rootProject.projectDir.parentFile
val rustTarget = System.getenv("CARGO_TARGET_DIR")?.let(::File) ?: File(rustRoot, "target")
val jniOut = layout.buildDirectory.dir("rust/jniLibs")
val bindingsOut = layout.buildDirectory.dir("generated/uniffi")
val cargo = if (System.getProperty("os.name").startsWith("Windows")) "cargo.exe" else "cargo"

android {
    namespace = "dev.rubi.app"
    compileSdk = 36
    buildToolsVersion = "36.1.0"
    ndkVersion = "29.0.14206865"

    defaultConfig {
        applicationId = "dev.rubi.app"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        ndk { abiFilters += "arm64-v8a" }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            // Self-signed builds only; there is no store release.
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures { compose = true }

    sourceSets["main"].jniLibs.srcDir(jniOut)
    sourceSets["main"].java.srcDir(bindingsOut)
}

kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }

/** Builds librubi_ffi.so for arm64 with cargo-ndk. */
val cargoBuild by tasks.registering(Exec::class) {
    workingDir = rustRoot
    commandLine(cargo, "ndk", "-t", "arm64-v8a", "-o", jniOut.get().asFile.absolutePath, "build", "--release", "-p", "rubi-ffi")
    inputs.dir(File(rustRoot, "crates"))
    inputs.file(File(rustRoot, "Cargo.lock"))
    outputs.dir(jniOut)
}

/** Generates the Kotlin bindings from the built library (UniFFI library mode). */
val uniffiBindgen by tasks.registering(Exec::class) {
    dependsOn(cargoBuild)
    workingDir = rustRoot
    val lib = File(rustTarget, "aarch64-linux-android/release/librubi_ffi.so")
    commandLine(
        cargo, "run", "-q", "-p", "rubi-ffi", "--features", "cli", "--bin", "uniffi-bindgen", "--",
        "generate", "--library", lib.absolutePath, "--language", "kotlin", "--no-format",
        "--out-dir", bindingsOut.get().asFile.absolutePath,
    )
    inputs.file(lib)
    outputs.dir(bindingsOut)
    doLast {
        val generated = bindingsOut.get().asFile.walkTopDown().any { it.extension == "kt" }
        check(generated) { "uniffi-bindgen produced no Kotlin; is the library stripped of UNIFFI_META symbols?" }
    }
}

tasks.named("preBuild") { dependsOn(uniffiBindgen) }

dependencies {
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.ui)
    implementation(libs.compose.foundation)
    implementation(libs.compose.material3)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.activity.compose)
    implementation(libs.lifecycle.runtime.compose)
    implementation(libs.media3.exoplayer)
    implementation(libs.media3.hls)
    implementation(libs.media3.ui)
    implementation(libs.media3.session)
    implementation(libs.coroutines.android)
    implementation("${libs.jna.get()}@aar")
    implementation(libs.coil.compose)
    implementation(libs.coil.network)
    implementation(libs.zxing.core)
}
