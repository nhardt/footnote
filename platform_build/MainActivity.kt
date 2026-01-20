package dev.dioxus.main

import android.content.Intent
import android.os.Bundle
import wiki.footnote.app.BuildConfig
import android.util.Log
import dev.dioxus.main.WryActivity

typealias BuildConfig = BuildConfig

class MainActivity : WryActivity() {
    private val TAG = "FootnoteKotlin"

    override fun onNewIntent(intent: Intent) {
        Log.d(TAG, "onNewIntent called")
        super.onNewIntent(intent)
        setIntent(intent)
        Log.d(TAG, "onNewIntent received: ${intent.action}")
        intent.data?.let { uri ->
            val uriString = uri.toString()
            Log.d(TAG, "Attempting JNI call with URI: $uriString")
            try {
                notifyOnNewIntent(uriString)
                Log.d(TAG, "JNI call successful")
            } catch (e: Exception) {
                Log.e(TAG, "JNI call failed!", e)
            }
        } ?: Log.w(TAG, "Intent data was null")
    }

    companion object {
        init {
            // Ensure this name matches your Cargo.toml [lib] name
            System.loadLibrary("dioxusmain")
        }
    }

    private external fun notifyOnNewIntent(uri: String)
}
