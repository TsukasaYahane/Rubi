package dev.rubi.app.cast

import java.io.BufferedInputStream
import java.io.OutputStream
import java.net.HttpURLConnection
import java.net.Inet4Address
import java.net.NetworkInterface
import java.net.ServerSocket
import java.net.Socket
import java.net.URL
import java.util.UUID
import java.util.concurrent.Executors
import java.util.concurrent.RejectedExecutionException
import kotlin.concurrent.thread

/**
 * The phone's side of 投屏: a small HTTP server on the Wi-Fi address that the TV fetches the
 * video from. Each request is fetched from B站 with the headers Rubi's own player uses (the TV
 * cannot send them) and passed on as it comes, ranges included, so the TV can seek. It serves
 * only the one current video, under an unguessable path, and stops when 投屏 ends.
 */
class CastRelay {
    @Volatile private var server: ServerSocket? = null
    /** A TV opens a few connections at once (probe, ranges); more wait, none starve the phone. */
    private val workers = Executors.newFixedThreadPool(MAX_CLIENTS) { r -> Thread(r, "rubi-cast-client").apply { isDaemon = true } }
    @Volatile private var source: List<String> = emptyList()
    @Volatile private var headers: Map<String, String> = emptyMap()
    @Volatile private var token = UUID.randomUUID().toString()

    /** Serves `urls` (the first that answers) and returns the address to give the TV. */
    fun serve(urls: List<String>, requestHeaders: Map<String, String>): String? {
        val ip = lanAddress() ?: return null
        source = urls
        headers = requestHeaders
        token = UUID.randomUUID().toString()
        val s = server ?: ServerSocket(0).also { srv ->
            server = srv
            thread(name = "rubi-cast-relay", isDaemon = true) {
                while (!srv.isClosed) {
                    val client = runCatching { srv.accept() }.getOrNull() ?: break
                    // A client that connects and says nothing is dropped instead of holding a worker.
                    runCatching { client.soTimeout = CLIENT_TIMEOUT_MS }
                    try {
                        workers.execute { runCatching { handle(client) }; runCatching { client.close() } }
                    } catch (e: RejectedExecutionException) {
                        runCatching { client.close() }
                    }
                }
            }
        }
        return "http://$ip:${s.localPort}/v/$token.mp4"
    }

    fun stop() {
        runCatching { server?.close() }
        server = null
        source = emptyList()
    }

    private fun handle(client: Socket) {
        val input = BufferedInputStream(client.getInputStream())
        val lines = generateSequence { readLine(input) }.takeWhile { it.isNotEmpty() }.take(MAX_HEADER_LINES).toList()
        val request = lines.firstOrNull() ?: return
        val method = request.substringBefore(' ')
        val path = request.split(' ').getOrNull(1) ?: ""
        val out = client.getOutputStream()
        if (path != "/v/$token.mp4" || (method != "GET" && method != "HEAD")) {
            out.write("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".toByteArray())
            return
        }
        val range = lines.firstOrNull { it.startsWith("range:", ignoreCase = true) }?.substringAfter(':')?.trim()
        for (url in source) {
            val conn = runCatching {
                (URL(url).openConnection() as HttpURLConnection).apply {
                    requestMethod = method
                    connectTimeout = 8_000
                    readTimeout = 15_000
                    headers.forEach { (k, v) -> setRequestProperty(k, v) }
                    range?.let { setRequestProperty("Range", it) }
                }
            }.getOrNull() ?: continue
            val code = runCatching { conn.responseCode }.getOrDefault(-1)
            if (code != 200 && code != 206) {
                conn.disconnect()
                continue
            }
            relay(conn, code, method, out)
            return
        }
        out.write("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".toByteArray())
    }

    private fun relay(conn: HttpURLConnection, code: Int, method: String, out: OutputStream) {
        try {
            val head = buildString {
                append(if (code == 206) "HTTP/1.1 206 Partial Content\r\n" else "HTTP/1.1 200 OK\r\n")
                append("Content-Type: video/mp4\r\nAccept-Ranges: bytes\r\nConnection: close\r\n")
                conn.getHeaderField("Content-Length")?.let { append("Content-Length: $it\r\n") }
                conn.getHeaderField("Content-Range")?.let { append("Content-Range: $it\r\n") }
                append("transferMode.dlna.org: Streaming\r\n\r\n")
            }
            out.write(head.toByteArray())
            if (method == "GET") conn.inputStream.use { it.copyTo(out, 64 * 1024) }
            out.flush()
        } finally {
            conn.disconnect()
        }
    }

    private fun readLine(input: BufferedInputStream): String? {
        val sb = StringBuilder()
        while (true) {
            val b = input.read()
            if (b < 0) return if (sb.isEmpty()) null else sb.toString()
            if (b == '\n'.code) return sb.toString().trimEnd('\r')
            sb.append(b.toChar())
            if (sb.length > 8_192) return sb.toString()
        }
    }

    /** The phone's IPv4 address on the local network (Wi-Fi). */
    private fun lanAddress(): String? = runCatching {
        NetworkInterface.getNetworkInterfaces().toList()
            .filter { it.isUp && !it.isLoopback }
            .sortedBy { if (it.name.startsWith("wlan")) 0 else 1 }
            .flatMap { it.inetAddresses.toList() }
            .firstOrNull { it is Inet4Address && it.isSiteLocalAddress }
            ?.hostAddress
    }.getOrNull()
}

/** Connections served at once. */
private const val MAX_CLIENTS = 4
/** A read that waits longer than this ends the connection. */
private const val CLIENT_TIMEOUT_MS = 30_000
/** Request header lines read at most. */
private const val MAX_HEADER_LINES = 100
