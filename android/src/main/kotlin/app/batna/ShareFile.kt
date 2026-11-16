package app.batna

import android.app.Activity
import android.content.ClipData
import android.content.ContentProvider
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import android.util.Log
import java.io.File
import java.io.FileNotFoundException

/**
 * Settings › Your data › the content-checks file: one tap hands it to the
 * system share sheet (Gmail, WhatsApp, Telegram, Drive…) with the file
 * attached. The file is copied into the app's cache (`share/`) and served
 * read-only by [ShareFileProvider], so no storage permission is needed and
 * nothing outside that folder is ever exposed. Rust calls [ShareFile.share].
 */
object ShareFile {
    private const val TAG = "BatnaShareFile"

    /** Copies the file at [path] and opens the share sheet for it. */
    @JvmStatic
    fun share(activity: Activity, path: String, mime: String, subject: String) {
        val source = File(path)
        val dir = File(activity.cacheDir, ShareFileProvider.DIR)
        dir.mkdirs()
        // One file at a time: an older copy is never offered again.
        dir.listFiles()?.forEach { it.delete() }
        val copy = File(dir, source.name)
        source.copyTo(copy, overwrite = true)
        val uri = Uri.Builder()
            .scheme("content")
            .authority(ShareFileProvider.authority(activity))
            .appendPath(copy.name)
            .build()
        val send = Intent(Intent.ACTION_SEND)
            .setType(mime)
            .putExtra(Intent.EXTRA_STREAM, uri)
            .putExtra(Intent.EXTRA_SUBJECT, subject)
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        // The chooser passes the read grant on only through ClipData.
        send.clipData = ClipData.newRawUri(copy.name, uri)
        activity.runOnUiThread {
            try {
                activity.startActivity(
                    Intent.createChooser(send, subject).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION),
                )
            } catch (e: Exception) {
                Log.w(TAG, "No share sheet", e)
            }
        }
    }
}

/** Serves the files in `cache/share/` by name, read-only, to the app the learner picked. */
class ShareFileProvider : ContentProvider() {
    override fun onCreate(): Boolean = true

    private fun file(uri: Uri): File {
        val context = context ?: throw FileNotFoundException("no context")
        val name = uri.lastPathSegment ?: throw FileNotFoundException("no name")
        val dir = File(context.cacheDir, DIR).canonicalFile
        val file = File(dir, name).canonicalFile
        // Only a plain file directly in share/: no "..", no other folder.
        if (file.parentFile != dir || !file.isFile) throw FileNotFoundException(name)
        return file
    }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor {
        if (mode != "r") throw SecurityException("read-only")
        return ParcelFileDescriptor.open(file(uri), ParcelFileDescriptor.MODE_READ_ONLY)
    }

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor {
        val f = file(uri)
        val columns = projection ?: arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE)
        val row = columns.map {
            when (it) {
                OpenableColumns.DISPLAY_NAME -> f.name
                OpenableColumns.SIZE -> f.length()
                else -> null
            }
        }
        return MatrixCursor(columns, 1).apply { addRow(row) }
    }

    override fun getType(uri: Uri): String =
        if (uri.lastPathSegment?.endsWith(".json") == true) "application/json" else "text/plain"

    override fun insert(uri: Uri, values: ContentValues?): Uri? = null

    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int = 0

    override fun update(uri: Uri, values: ContentValues?, selection: String?, selectionArgs: Array<out String>?): Int = 0

    companion object {
        const val DIR = "share"

        fun authority(context: Context) = context.packageName + ".share"
    }
}
