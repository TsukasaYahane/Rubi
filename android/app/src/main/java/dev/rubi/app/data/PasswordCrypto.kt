package dev.rubi.app.data

import android.util.Base64
import java.security.KeyFactory
import java.security.spec.X509EncodedKeySpec
import javax.crypto.Cipher

/**
 * Encrypts the login password the way the web login expects: RSA (PKCS#1 v1.5) over
 * `hash + password` with the server's public key, base64. Done here so the plain password never
 * crosses into the Rust core; it is neither stored nor logged.
 */
object PasswordCrypto {
    fun encrypt(hash: String, publicKeyPem: String, password: String): String {
        val body = publicKeyPem.replace(Regex("-----[A-Z ]+-----"), "").replace(Regex("\\s"), "")
        val key = KeyFactory.getInstance("RSA").generatePublic(X509EncodedKeySpec(Base64.decode(body, Base64.DEFAULT)))
        val cipher = Cipher.getInstance("RSA/ECB/PKCS1Padding").apply { init(Cipher.ENCRYPT_MODE, key) }
        return Base64.encodeToString(cipher.doFinal((hash + password).toByteArray(Charsets.UTF_8)), Base64.NO_WRAP)
    }
}
