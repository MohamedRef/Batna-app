//! Local notifications for the daily study reminder, through
//! `app.batna.ReminderBridge` (android/). Android only for now; elsewhere
//! every call is a no-op and [`LOCAL`] is false, so Settings hides the rows.
// Adapted from kabar-app main: crates/app/src/platform/android.rs (reminders).

/// This platform arms reminders itself.
pub const LOCAL: bool = cfg!(target_os = "android");

/// Replaces the armed reminders with `json` (`[]` clears them).
pub fn schedule(json: &str) {
    #[cfg(target_os = "android")]
    report(android::schedule(json));
    #[cfg(not(target_os = "android"))]
    let _ = json;
}

/// The fallback the platform arms once the planned reminders run out
/// (`plan::Repeat` as JSON, `{}` for none).
pub fn repeat(json: &str) {
    #[cfg(target_os = "android")]
    report(android::string_call("repeat", json));
    #[cfg(not(target_os = "android"))]
    let _ = json;
}

/// Whether notifications are allowed for Batna (None where unknown).
pub fn enabled() -> Option<bool> {
    #[cfg(target_os = "android")]
    return android::flag("enabled").map_err(|e| crate::diag::report!("ReminderBridge: {e}")).ok();
    #[cfg(not(target_os = "android"))]
    None
}

/// Whether One UI lets Batna's alarms run while it sleeps (None where unknown).
pub fn battery_unrestricted() -> Option<bool> {
    #[cfg(target_os = "android")]
    return android::flag("batteryUnrestricted")
        .map_err(|e| crate::diag::report!("ReminderBridge: {e}"))
        .ok();
    #[cfg(not(target_os = "android"))]
    None
}

/// Android 13+: the system prompt for notifications, if not granted yet.
pub fn request_permission() {
    #[cfg(target_os = "android")]
    report(android::activity_call("requestPermission"));
}

/// Batna's notification page in Android's settings.
pub fn open_notification_settings() {
    #[cfg(target_os = "android")]
    report(android::activity_call("openNotificationSettings"));
}

/// Batna's app page in Android's settings (battery: Unrestricted).
pub fn open_app_settings() {
    #[cfg(target_os = "android")]
    report(android::activity_call("openAppSettings"));
}

#[cfg(target_os = "android")]
fn report(result: Result<(), String>) {
    if let Err(e) = result {
        crate::diag::report!("ReminderBridge: {e}");
    }
}

#[cfg(target_os = "android")]
mod android {
    use jni::objects::{JClass, JClassLoader, JObject};
    use jni::refs::LoaderContext;
    use jni::strings::JNIStr;
    use jni::{Env, JValue, jni_sig, jni_str};

    use crate::platform::tts_android::with_activity;

    /// `app.batna.ReminderBridge`, loaded through the Activity's class loader.
    fn bridge<T>(f: impl FnOnce(&mut Env, &JClass, &JObject) -> jni::errors::Result<T>) -> Result<T, String> {
        with_activity(|env, activity| {
            let loader = env
                .call_method(
                    activity,
                    jni_str!("getClassLoader"),
                    jni_sig!("()Ljava/lang/ClassLoader;"),
                    &[],
                )?
                .l()?;
            let loader = env.cast_local::<JClassLoader>(loader)?;
            let class =
                LoaderContext::Loader(&loader).load_class(env, jni_str!("app.batna.ReminderBridge"), true)?;
            f(env, &class, activity)
        })
    }

    pub fn schedule(json: &str) -> Result<(), String> {
        string_call("schedule", json)
    }

    /// A `(Context, String) -> Unit` method.
    pub fn string_call(name: &'static str, json: &str) -> Result<(), String> {
        bridge(|env, class, activity| {
            let json = env.new_string(json)?;
            env.call_static_method(
                class,
                method(name),
                jni_sig!("(Landroid/content/Context;Ljava/lang/String;)V"),
                &[JValue::Object(activity), JValue::Object(&json)],
            )
            .map(drop)
        })
    }

    /// A `(Context) -> Boolean` method.
    pub fn flag(name: &'static str) -> Result<bool, String> {
        bridge(|env, class, activity| {
            let name = method(name);
            env.call_static_method(
                class,
                name,
                jni_sig!("(Landroid/content/Context;)Z"),
                &[JValue::Object(activity)],
            )?
            .z()
        })
    }

    /// A `(Activity) -> Unit` method.
    pub fn activity_call(name: &'static str) -> Result<(), String> {
        bridge(|env, class, activity| {
            env.call_static_method(
                class,
                method(name),
                jni_sig!("(Landroid/app/Activity;)V"),
                &[JValue::Object(activity)],
            )
            .map(drop)
        })
    }

    fn method(name: &'static str) -> &'static JNIStr {
        match name {
            "schedule" => jni_str!("schedule"),
            "repeat" => jni_str!("repeat"),
            "enabled" => jni_str!("enabled"),
            "batteryUnrestricted" => jni_str!("batteryUnrestricted"),
            "requestPermission" => jni_str!("requestPermission"),
            "openNotificationSettings" => jni_str!("openNotificationSettings"),
            _ => jni_str!("openAppSettings"),
        }
    }
}
