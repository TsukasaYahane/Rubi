package dev.rubi.app.ui

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * Runs a core call that reads or writes the local database or files (play lists, downloads,
 * the feed pool) off the main thread, so a slow disk never stalls a frame.
 */
suspend fun <T> offMain(block: suspend () -> T): T = withContext(Dispatchers.IO) { block() }
