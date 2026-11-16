//! One tap from a file Batna just wrote to the system share menu, with the
//! file attached: the share sheet on Android (`app.batna.ShareFile` in
//! android/) and the share menu on the Mac (`NSSharingServicePicker`).
//! Elsewhere (the web, Linux, tests) [`available`] is false and the caller
//! keeps its plain "save the file" button.

/// Whether this platform has a share menu Batna can open.
pub fn available() -> bool {
    cfg!(any(target_os = "android", target_os = "macos")) && !cfg!(test)
}

/// Opens the share menu for the file at `path`; `subject` titles the menu
/// and becomes the e-mail subject where the target uses one.
pub fn file(path: &str, mime: &str, subject: &str) -> Result<(), String> {
    #[cfg(all(target_os = "android", not(test)))]
    return android::share(path, mime, subject);
    #[cfg(all(target_os = "macos", not(test)))]
    {
        let _ = (mime, subject);
        return mac::share(path);
    }
    #[allow(unreachable_code)]
    {
        let _ = (path, mime, subject);
        Err("no share menu here".into())
    }
}

#[cfg(all(target_os = "android", not(test)))]
mod android {
    use jni::objects::{JClassLoader, JValue};
    use jni::refs::LoaderContext;
    use jni::{jni_sig, jni_str};

    use crate::platform::tts_android::with_activity;

    pub fn share(path: &str, mime: &str, subject: &str) -> Result<(), String> {
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
                LoaderContext::Loader(&loader).load_class(env, jni_str!("app.batna.ShareFile"), true)?;
            let (path, mime, subject) =
                (env.new_string(path)?, env.new_string(mime)?, env.new_string(subject)?);
            env.call_static_method(
                &class,
                jni_str!("share"),
                jni_sig!("(Landroid/app/Activity;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
                &[
                    JValue::Object(activity),
                    JValue::Object(&path),
                    JValue::Object(&mime),
                    JValue::Object(&subject),
                ],
            )
            .map(drop)
        })
    }
}

#[cfg(all(target_os = "macos", not(test)))]
mod mac {
    use objc2::MainThreadMarker;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSApplication, NSSharingServicePicker};
    use objc2_foundation::{NSArray, NSRectEdge, NSString, NSURL};

    /// The share menu under the middle of the window's top edge, where the
    /// settings sheet's card sits on every width.
    pub fn share(path: &str) -> Result<(), String> {
        let mtm = MainThreadMarker::new().ok_or("not on the main thread")?;
        let app = NSApplication::sharedApplication(mtm);
        let window = app.keyWindow().or_else(|| app.mainWindow()).ok_or("no window")?;
        let view = window.contentView().ok_or("no view")?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path));
        let items: Retained<NSArray<AnyObject>> =
            NSArray::from_retained_slice(&[Retained::into_super(Retained::into_super(url))]);
        let picker = unsafe { NSSharingServicePicker::initWithItems(mtm.alloc(), &items) };
        let mut rect = view.bounds();
        rect.origin.x += rect.size.width / 2.0;
        rect.size.width = 1.0;
        rect.size.height = 1.0;
        picker.showRelativeToRect_ofView_preferredEdge(rect, &view, NSRectEdge::MinY);
        Ok(())
    }
}
