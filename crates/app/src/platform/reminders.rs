//! Local notifications for the daily study reminder: through
//! `app.batna.ReminderBridge` (android/) on Android, and
//! UNUserNotificationCenter from the bundled Batna.app on the Mac. Elsewhere
//! (and from `cargo run`, which has no bundle) every call is a no-op and
//! [`local`] is false, so Settings hides the rows.
// Adapted from kabar-app main: crates/app/src/platform/android.rs and
// crates/app/src/platform/macos.rs (reminders).

/// This device can arm reminders itself.
pub fn local() -> bool {
    #[cfg(target_os = "macos")]
    return mac::bundled();
    #[cfg(not(target_os = "macos"))]
    cfg!(target_os = "android")
}

/// Replaces the armed reminders with `json` (`[]` clears them).
pub fn schedule(json: &str) {
    #[cfg(target_os = "android")]
    report(android::schedule(json));
    #[cfg(target_os = "macos")]
    if let Err(e) = mac::schedule(json) {
        eprintln!("Mac reminders: {e}");
    }
    #[cfg(not(any(target_os = "android", target_os = "macos")))]
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
    return android::flag("enabled").map_err(|e| eprintln!("ReminderBridge: {e}")).ok();
    #[cfg(not(target_os = "android"))]
    None
}

/// Whether One UI lets Batna's alarms run while it sleeps (None where unknown).
pub fn battery_unrestricted() -> Option<bool> {
    #[cfg(target_os = "android")]
    return android::flag("batteryUnrestricted").map_err(|e| eprintln!("ReminderBridge: {e}")).ok();
    #[cfg(not(target_os = "android"))]
    None
}

/// The system prompt for notifications, if not answered yet (Android 13+,
/// and the Mac).
pub fn request_permission() {
    #[cfg(target_os = "android")]
    report(android::activity_call("requestPermission"));
    #[cfg(target_os = "macos")]
    mac::request();
}

/// Batna's notification page in the system settings.
pub fn open_notification_settings() {
    #[cfg(target_os = "android")]
    report(android::activity_call("openNotificationSettings"));
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.Notifications-Settings.extension")
        .spawn();
}

/// Batna's app page in Android's settings (battery: Unrestricted).
pub fn open_app_settings() {
    #[cfg(target_os = "android")]
    report(android::activity_call("openAppSettings"));
}

#[cfg(target_os = "android")]
fn report(result: Result<(), String>) {
    if let Err(e) = result {
        eprintln!("ReminderBridge: {e}");
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

/// The bundled Mac app's notifications. UNUserNotificationCenter needs a real
/// bundle (`app.batna`): from `cargo run` there is none and the framework
/// would abort, so everything checks for one first. Calendar triggers fire at
/// the local date and time in the Mac's own zone, so daylight saving never
/// moves a reminder.
#[cfg(target_os = "macos")]
mod mac {
    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSBundle, NSDateComponents, NSError, NSString};
    use objc2_user_notifications::{
        UNAuthorizationOptions, UNCalendarNotificationTrigger, UNMutableNotificationContent,
        UNNotificationRequest, UNUserNotificationCenter,
    };
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Item {
        date: String,
        time: String,
        title: String,
        text: String,
    }

    pub fn bundled() -> bool {
        NSBundle::mainBundle().bundleIdentifier().is_some()
    }

    pub fn request() {
        if !bundled() {
            return;
        }
        let done = RcBlock::new(|granted: Bool, _error: *mut NSError| {
            if !granted.as_bool() {
                eprintln!("notifications not allowed on this Mac");
            }
        });
        UNUserNotificationCenter::currentNotificationCenter()
            .requestAuthorizationWithOptions_completionHandler(
                UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
                &done,
            );
    }

    /// Replaces Batna's pending notifications with `json`'s.
    pub fn schedule(json: &str) -> Result<(), String> {
        if !bundled() {
            return Ok(());
        }
        let items: Vec<Item> = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let center = UNUserNotificationCenter::currentNotificationCenter();
        center.removeAllPendingNotificationRequests();
        for (i, item) in items.iter().enumerate() {
            let (Some((y, mo, d)), Some((h, mi))) = (ymd(&item.date), hm(&item.time)) else { continue };
            let when = NSDateComponents::new();
            when.setYear(y);
            when.setMonth(mo);
            when.setDay(d);
            when.setHour(h);
            when.setMinute(mi);
            let trigger =
                UNCalendarNotificationTrigger::triggerWithDateMatchingComponents_repeats(&when, false);
            let content = UNMutableNotificationContent::new();
            content.setTitle(&NSString::from_str(&item.title));
            content.setBody(&NSString::from_str(&item.text));
            let id = NSString::from_str(&format!("batna.reminder.{i}"));
            let request =
                UNNotificationRequest::requestWithIdentifier_content_trigger(&id, &content, Some(&trigger));
            center.addNotificationRequest_withCompletionHandler(&request, None);
        }
        Ok(())
    }

    fn ymd(s: &str) -> Option<(isize, isize, isize)> {
        let mut parts = s.split('-').map(|p| p.parse().ok());
        Some((parts.next()??, parts.next()??, parts.next()??))
    }

    fn hm(s: &str) -> Option<(isize, isize)> {
        let (h, m) = s.split_once(':')?;
        Some((h.parse().ok()?, m.parse().ok()?))
    }
}
