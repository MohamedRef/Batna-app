package app.batna

import android.Manifest
import android.app.Activity
import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.PowerManager
import android.provider.Settings
import org.json.JSONArray
import java.time.LocalDate
import java.time.LocalTime
import java.time.ZoneId

/**
 * Batna's daily study reminders, adapted from kabar-app main:
 * android/src/main/kotlin/app/kabar/Reminders.kt. Rust plans the next few
 * (`crates/app/src/reminders/plan.rs`) and hands them over as JSON; each
 * becomes an inexact [AlarmManager] alarm (no exact-alarm permission) that
 * [ReminderReceiver] posts. Offline, no server.
 *
 * The schedule is replaced, never added to, every time the app re-plans, so
 * a met target silences the rest of the day once the app has seen it. It is
 * kept in SharedPreferences to survive reboots and updates. Alarms are armed
 * from each reminder's local date and time in the device's zone, and re-armed
 * when the zone or the clock changes, so Egypt's daylight saving never moves
 * 19:00.
 */
object ReminderBridge {
    const val CHANNEL = "study"
    private const val PREFS = "batna_reminders"
    private const val KEY = "schedule"
    /** Alarms are request codes 0 until this; Rust never sends more. */
    const val MAX = 8
    private const val PERMISSION_REQUEST = 4120

    /**
     * `[{"at": unix ms, "date": "YYYY-MM-DD", "time": "HH:MM", "title", "text"}]`,
     * soonest first. `[]` clears them (another device rings, or reminders off).
     */
    @JvmStatic
    fun schedule(context: Context, json: String) {
        val app = context.applicationContext
        app.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(KEY, json).apply()
        arm(app, json)
    }

    /** Re-arms the stored schedule (after a reboot, an update or a clock change). */
    fun rearm(context: Context) {
        val json = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY, null) ?: return
        arm(context, json)
    }

    private fun arm(context: Context, json: String) {
        val alarms = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
        for (i in 0 until MAX) {
            alarms.cancel(pending(context, i, null, null))
        }
        val items = try { JSONArray(json) } catch (e: org.json.JSONException) { JSONArray() }
        val now = System.currentTimeMillis()
        for (i in 0 until minOf(items.length(), MAX)) {
            val item = items.getJSONObject(i)
            val at = wallClock(item.optString("date"), item.optString("time")) ?: item.getLong("at")
            if (at <= now) continue
            val intent = pending(context, i, item.getString("title"), item.getString("text"))
            // Inexact while idle: Doze may hold it a few minutes, which suits a nudge.
            alarms.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, at, intent)
        }
    }

    /** The local date and time in the device's zone now, as unix ms. */
    private fun wallClock(date: String, time: String): Long? = try {
        LocalDate.parse(date).atTime(LocalTime.parse(time))
            .atZone(ZoneId.systemDefault()).toInstant().toEpochMilli()
    } catch (e: Exception) {
        null
    }

    private fun pending(context: Context, code: Int, title: String?, text: String?): PendingIntent {
        val intent = Intent(context, ReminderReceiver::class.java).setAction("app.batna.REMIND")
        title?.let { intent.putExtra("title", it) }
        text?.let { intent.putExtra("text", it) }
        return PendingIntent.getBroadcast(
            context, code, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
    }

    /** Whether reminders can show: the app's notifications are on. */
    @JvmStatic
    fun enabled(context: Context): Boolean =
        (context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager).areNotificationsEnabled()

    /** Shows the system prompt for notifications on Android 13+, if not granted yet. */
    @JvmStatic
    fun requestPermission(activity: Activity) {
        if (Build.VERSION.SDK_INT >= 33 &&
            activity.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) {
            activity.requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), PERMISSION_REQUEST)
        }
    }

    /** Opens Batna's notification page in Android's settings (after a "don't ask again"). */
    @JvmStatic
    fun openNotificationSettings(activity: Activity) {
        val intent = Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
            .putExtra(Settings.EXTRA_APP_PACKAGE, activity.packageName)
        try {
            activity.startActivity(intent)
        } catch (e: Exception) {
            openAppSettings(activity)
        }
    }

    /** Opens Batna's app page (battery: "Unrestricted" keeps One UI from sleeping its alarms). */
    @JvmStatic
    fun openAppSettings(activity: Activity) {
        activity.startActivity(
            Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", activity.packageName, null)),
        )
    }

    /**
     * Whether Android lets Batna run its alarms freely. Samsung's One UI puts
     * apps unused for a few days to sleep, and their alarms stop.
     */
    @JvmStatic
    fun batteryUnrestricted(context: Context): Boolean {
        val pm = context.getSystemService(Context.POWER_SERVICE) as PowerManager
        return pm.isIgnoringBatteryOptimizations(context.packageName)
    }

    fun show(context: Context, title: String, text: String) {
        val nm = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        nm.createNotificationChannel(
            NotificationChannel(CHANNEL, "تذكير المذاكرة", NotificationManager.IMPORTANCE_DEFAULT).apply {
                description = "تذكير هادي بمراجعة اليوم"
            },
        )
        val open = context.packageManager.getLaunchIntentForPackage(context.packageName)
            ?.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED)
        val tap = open?.let {
            PendingIntent.getActivity(context, 0, it, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        }
        val note = Notification.Builder(context, CHANNEL)
            // A system icon: Batna ships no image assets.
            .setSmallIcon(android.R.drawable.ic_popup_reminder)
            .setContentTitle(title)
            .setContentText(text)
            .setStyle(Notification.BigTextStyle().bigText(text))
            .setContentIntent(tap)
            .setAutoCancel(true)
            .build()
        // One reminder at a time: a newer one replaces an unread older one.
        nm.notify(CHANNEL.hashCode(), note)
    }
}

/** Posts a scheduled reminder, and re-arms the schedule after a reboot, an update or a clock change. */
class ReminderReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED,
            Intent.ACTION_TIMEZONE_CHANGED, Intent.ACTION_TIME_CHANGED -> ReminderBridge.rearm(context)
            else -> {
                val title = intent.getStringExtra("title") ?: return
                val text = intent.getStringExtra("text") ?: return
                if (ReminderBridge.enabled(context)) ReminderBridge.show(context, title, text)
            }
        }
    }
}
