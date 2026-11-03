package dev.rubi.app.cast

import android.content.Context
import android.net.wifi.WifiManager
import java.io.ByteArrayOutputStream
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketTimeoutException
import java.net.URI
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** A TV (or other player) on the network that takes media by DLNA. */
data class Renderer(val name: String, val avTransport: URI, val rendering: URI?)

/**
 * DLNA, as much as 投屏 needs: finding players on the Wi-Fi (SSDP), and telling one what to play,
 * to pause, seek or change volume (UPnP AVTransport / RenderingControl over SOAP). Requests go
 * over plain sockets: the player is a LAN address, not a site.
 */
object Dlna {
    private const val SSDP_HOST = "239.255.255.250"
    private const val SSDP_PORT = 1900
    private const val AV_TRANSPORT = "urn:schemas-upnp-org:service:AVTransport:1"
    private const val RENDERING = "urn:schemas-upnp-org:service:RenderingControl:1"
    private const val TIMEOUT_MS = 4_000

    /** Players that answer within `waitMs`, each once. */
    suspend fun discover(context: Context, waitMs: Int = 3_000): List<Renderer> = withContext(Dispatchers.IO) {
        val wifi = context.applicationContext.getSystemService(WifiManager::class.java)
        val lock = wifi?.createMulticastLock("rubi-cast")?.apply { setReferenceCounted(false); acquire() }
        try {
            val locations = linkedSetOf<String>()
            DatagramSocket().use { socket ->
                socket.soTimeout = 500
                val search = "M-SEARCH * HTTP/1.1\r\nHOST: $SSDP_HOST:$SSDP_PORT\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: $AV_TRANSPORT\r\n\r\n".toByteArray()
                val to = InetAddress.getByName(SSDP_HOST)
                repeat(2) { socket.send(DatagramPacket(search, search.size, to, SSDP_PORT)) }
                val until = System.currentTimeMillis() + waitMs
                val buf = ByteArray(2048)
                while (System.currentTimeMillis() < until) {
                    val p = DatagramPacket(buf, buf.size)
                    try {
                        socket.receive(p)
                    } catch (_: SocketTimeoutException) {
                        continue
                    }
                    val text = String(p.data, 0, p.length)
                    Regex("(?im)^location:\\s*(\\S+)").find(text)?.groupValues?.get(1)?.let { locations.add(it) }
                }
            }
            locations.mapNotNull { runCatching { describe(URI(it)) }.getOrNull() }.distinctBy { it.avTransport }
        } finally {
            lock?.release()
        }
    }

    /** Reads a device description: its name and the control addresses of the two services. */
    private fun describe(location: URI): Renderer? {
        val xml = http("GET", location, emptyMap(), null)
        val name = tag(xml, "friendlyName") ?: location.host
        val base = tag(xml, "URLBase")?.let { URI(it) } ?: location
        fun control(type: String): URI? = Regex("<service>([\\s\\S]*?)</service>").findAll(xml)
            .map { it.groupValues[1] }
            .firstOrNull { tag(it, "serviceType")?.startsWith(type.substringBeforeLast(':')) == true }
            ?.let { tag(it, "controlURL") }
            ?.let { base.resolve(it) }
        val av = control(AV_TRANSPORT) ?: return null
        return Renderer(name, av, control(RENDERING))
    }

    /** Hands `url` to the player and starts it. */
    suspend fun play(r: Renderer, url: String, title: String) = withContext(Dispatchers.IO) {
        val meta = "<DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" " +
            "xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\"><item id=\"0\" parentID=\"-1\" restricted=\"1\">" +
            "<dc:title>${escape(title)}</dc:title><upnp:class>object.item.videoItem</upnp:class>" +
            "<res protocolInfo=\"http-get:*:video/mp4:*\">${escape(url)}</res></item></DIDL-Lite>"
        soap(r.avTransport, AV_TRANSPORT, "SetAVTransportURI", "<InstanceID>0</InstanceID><CurrentURI>${escape(url)}</CurrentURI><CurrentURIMetaData>${escape(meta)}</CurrentURIMetaData>")
        soap(r.avTransport, AV_TRANSPORT, "Play", "<InstanceID>0</InstanceID><Speed>1</Speed>")
    }

    suspend fun resume(r: Renderer) = withContext(Dispatchers.IO) { soap(r.avTransport, AV_TRANSPORT, "Play", "<InstanceID>0</InstanceID><Speed>1</Speed>") }

    suspend fun pause(r: Renderer) = withContext(Dispatchers.IO) { soap(r.avTransport, AV_TRANSPORT, "Pause", "<InstanceID>0</InstanceID>") }

    suspend fun stop(r: Renderer) = withContext(Dispatchers.IO) { soap(r.avTransport, AV_TRANSPORT, "Stop", "<InstanceID>0</InstanceID>") }

    suspend fun seek(r: Renderer, seconds: Long) = withContext(Dispatchers.IO) {
        soap(r.avTransport, AV_TRANSPORT, "Seek", "<InstanceID>0</InstanceID><Unit>REL_TIME</Unit><Target>${clock(seconds)}</Target>")
    }

    /** Where the player is and how long the video is, in seconds. */
    suspend fun position(r: Renderer): Pair<Long, Long> = withContext(Dispatchers.IO) {
        val x = soap(r.avTransport, AV_TRANSPORT, "GetPositionInfo", "<InstanceID>0</InstanceID>")
        seconds(tag(x, "RelTime")) to seconds(tag(x, "TrackDuration"))
    }

    suspend fun volume(r: Renderer): Int? = withContext(Dispatchers.IO) {
        val rc = r.rendering ?: return@withContext null
        tag(soap(rc, RENDERING, "GetVolume", "<InstanceID>0</InstanceID><Channel>Master</Channel>"), "CurrentVolume")?.toIntOrNull()
    }

    suspend fun setVolume(r: Renderer, v: Int) = withContext(Dispatchers.IO) {
        val rc = r.rendering ?: return@withContext ""
        soap(rc, RENDERING, "SetVolume", "<InstanceID>0</InstanceID><Channel>Master</Channel><DesiredVolume>${v.coerceIn(0, 100)}</DesiredVolume>")
    }

    private fun soap(url: URI, service: String, action: String, args: String): String {
        val body = "<?xml version=\"1.0\" encoding=\"utf-8\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" " +
            "s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:$action xmlns:u=\"$service\">$args</u:$action></s:Body></s:Envelope>"
        return http("POST", url, mapOf("Content-Type" to "text/xml; charset=\"utf-8\"", "SOAPACTION" to "\"$service#$action\""), body)
    }

    /** A minimal HTTP/1.1 exchange over a socket; returns the body. */
    private fun http(method: String, url: URI, headers: Map<String, String>, body: String?): String {
        val port = if (url.port > 0) url.port else 80
        val path = (url.rawPath ?: "/").ifEmpty { "/" } + (url.rawQuery?.let { "?$it" } ?: "")
        Socket().use { s ->
            s.connect(InetSocketAddress(url.host, port), TIMEOUT_MS)
            s.soTimeout = TIMEOUT_MS
            val payload = body?.toByteArray() ?: ByteArray(0)
            val head = buildString {
                append("$method $path HTTP/1.1\r\nHost: ${url.host}:$port\r\nConnection: close\r\nUser-Agent: Rubi/DLNA\r\n")
                headers.forEach { (k, v) -> append("$k: $v\r\n") }
                if (body != null) append("Content-Length: ${payload.size}\r\n")
                append("\r\n")
            }
            s.getOutputStream().apply { write(head.toByteArray()); write(payload); flush() }
            val out = ByteArrayOutputStream()
            s.getInputStream().copyTo(out)
            val text = out.toString(Charsets.UTF_8.name())
            val status = text.substringBefore("\r\n")
            val content = text.substringAfter("\r\n\r\n", "")
            if (!status.contains(" 200")) throw IllegalStateException("$status ${tag(content, "errorDescription") ?: ""}".trim())
            return content
        }
    }

    private fun tag(xml: String, name: String): String? =
        Regex("<(?:\\w+:)?$name[^>]*>([\\s\\S]*?)</(?:\\w+:)?$name>").find(xml)?.groupValues?.get(1)?.trim()

    private fun escape(s: String) = s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\"", "&quot;")

    private fun clock(s: Long) = "%02d:%02d:%02d".format(s / 3600, s / 60 % 60, s % 60)

    private fun seconds(t: String?): Long {
        val parts = t?.substringBefore('.')?.split(':')?.mapNotNull { it.toLongOrNull() } ?: return 0
        return parts.fold(0L) { acc, p -> acc * 60 + p }
    }
}
