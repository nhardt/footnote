use jni::objects::JValue;
use jni::{objects::JObject, JNIEnv, JavaVM};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::sync::OnceLock;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

// We store the JVM globally so any thread can "attach" to it later
static JVM: OnceLock<JavaVM> = OnceLock::new();
static FILE_SENDER: OnceLock<UnboundedSender<String>> = OnceLock::new();
static RECEIVER_STORAGE: OnceLock<std::sync::Mutex<Option<UnboundedReceiver<String>>>> =
    OnceLock::new();

fn init_channel() {
    RECEIVER_STORAGE.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded_channel();
        FILE_SENDER.set(tx).expect("sender already set");
        std::sync::Mutex::new(Some(rx))
    });
}

pub fn take_file_receiver() -> Option<UnboundedReceiver<String>> {
    init_channel();
    RECEIVER_STORAGE.get()?.lock().ok()?.take()
}

pub fn send_incoming_file(uri_or_path: String) {
    init_channel();
    if let Some(tx) = FILE_SENDER.get() {
        let _ = tx.send(uri_or_path);
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_dioxus_main_MainActivity_notifyOnNewIntent(
    mut env: jni::JNIEnv,
    _class: jni::objects::JClass,
    data: jni::objects::JString,
) {
    if let Ok(uri) = env.get_string(&data) {
        let uri_str: String = uri.into();
        send_incoming_file(uri_str);
    }
}

pub fn with_android_context<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut JNIEnv, &JObject) -> Option<R>,
{
    // 1. Get the VM (initialize it if this is the first time)
    let vm = JVM.get_or_init(|| {
        let ctx = ndk_context::android_context();
        unsafe { JavaVM::from_raw(ctx.vm().cast()).expect("Failed to get JVM") }
    });

    // 2. Attach the current thread to the JVM
    // This is vital for Dioxus because Dioxus often runs on its own threads
    let mut env = vm.attach_current_thread().ok()?;

    // 3. Get the Activity context
    let ctx = ndk_context::android_context();
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    // 4. Run your logic
    f(&mut env, &activity)
}

struct AndroidConstants {
    flag_read: i32,
    flag_new_task: i32,
}

impl AndroidConstants {
    fn fetch(env: &mut jni::JNIEnv) -> Option<Self> {
        let cls = env.find_class("android/content/Intent").ok()?;
        Some(Self {
            flag_read: env
                .get_static_field(&cls, "FLAG_GRANT_READ_URI_PERMISSION", "I")
                .ok()?
                .i()
                .ok()?,
            flag_new_task: env
                .get_static_field(&cls, "FLAG_ACTIVITY_NEW_TASK", "I")
                .ok()?
                .i()
                .ok()?,
        })
    }
}

pub fn handle_incoming_share() -> Result<Option<String>, String> {
    with_android_context(|env, activity| {
        tracing::debug!("getting the intent");
        let intent = env
            .call_method(activity, "getIntent", "()Landroid/content/Intent;", &[])
            .map_err(|e| format!("JNI Error getting intent: {:?}", e))
            .ok()?
            .l()
            .ok()?;
        if intent.is_null() || is_intent_processed(env, &intent) {
            return Some(Ok(None));
        }

        // Check if the intent contains a direct URL string (Deep Link)
        let data_uri_obj = env
            .call_method(&intent, "getDataString", "()Ljava/lang/String;", &[])
            .ok()?
            .l()
            .ok()?;

        if !data_uri_obj.is_null() {
            let data_uri: String = env.get_string(&data_uri_obj.into()).ok()?.into();
            if data_uri.starts_with("footnote+pair://") {
                mark_intent_processed(env, &intent);
                return Some(Ok(Some(data_uri)));
            }
        }

        tracing::debug!("getting the action");
        let action_obj = env
            .call_method(&intent, "getAction", "()Ljava/lang/String;", &[])
            .map_err(|e| format!("JNI Error getting action: {:?}", e))
            .ok()?
            .l()
            .ok()?;
        if action_obj.is_null() {
            return Some(Ok(None));
        }
        let action_str: String = env.get_string(&action_obj.into()).ok()?.into();
        if action_str != "android.intent.action.SEND" && action_str != "android.intent.action.VIEW"
        {
            return Some(Ok(None));
        }

        match read_content_uri(env, activity, &intent, &action_str) {
            Some(data) => {
                mark_intent_processed(env, &intent);
                Some(Ok(Some(data)))
            }
            None => {
                // Check if we exited read_content_uri because of a Java Exception
                if env.exception_check().unwrap_or(false) {
                    let _ = env.exception_describe(); // Prints to Logcat
                    let _ = env.exception_clear();
                    return Some(Err(
                        "Java Exception while reading URI (Security?)".to_string()
                    ));
                }
                Some(Err("Failed to read URI content".to_string()))
            }
        }
    })
    .unwrap_or(Ok(None))
}

fn mark_intent_processed(env: &mut JNIEnv, intent: &JObject) {
    let mut mark = || -> jni::errors::Result<()> {
        let key = env.new_string("processed")?;
        env.call_method(
            intent,
            "putExtra",
            "(Ljava/lang/String;Z)Landroid/content/Intent;",
            &[JValue::Object(&key), JValue::Bool(true.into())],
        )?;
        Ok(())
    };

    let _ = mark(); // We ignore errors here as it's a "best effort" cleanup
}

fn is_intent_processed(env: &mut JNIEnv, intent: &JObject) -> bool {
    let mut check = || -> jni::errors::Result<bool> {
        let key = env.new_string("processed")?;
        let processed = env
            .call_method(
                intent,
                "getBooleanExtra",
                "(Ljava/lang/String;Z)Z",
                &[JValue::Object(&key), JValue::Bool(false.into())],
            )?
            .z()?;
        Ok(processed)
    };

    check().unwrap_or(false)
}

pub fn read_uri_from_string(uri_str: String) -> Option<String> {
    with_android_context(|env, activity| {
        let uri_cls = env.find_class("android/net/Uri").ok()?;
        let j_uri_str = env.new_string(&uri_str).ok()?;
        let uri_obj = env
            .call_static_method(
                uri_cls,
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[jni::objects::JValue::Object(&j_uri_str)],
            )
            .ok()?
            .l()
            .ok()?;

        let resolver = env
            .call_method(
                activity,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[],
            )
            .ok()?
            .l()
            .ok()?;

        let input_stream_result = env.call_method(
            &resolver,
            "openInputStream",
            "(Landroid/net/Uri;)Ljava/io/InputStream;",
            &[JValue::Object(&uri_obj)],
        );

        if env.exception_check().ok().unwrap_or(false) {
            env.exception_describe().ok();
            env.exception_clear().ok();
            return None;
        }

        let input_stream = input_stream_result.ok()?.l().ok()?;

        perform_stream_read(env, &input_stream)
    })
}

fn read_content_uri(
    env: &mut JNIEnv,
    activity: &JObject,
    intent: &JObject,
    action: &str,
) -> Option<String> {
    tracing::debug!("attempting to read content uri");

    let uri = if action == "android.intent.action.SEND" {
        tracing::debug!("getting uri via extra");
        let extra_key = env
            .get_static_field(
                "android/content/Intent",
                "EXTRA_STREAM",
                "Ljava/lang/String;",
            )
            .ok()?
            .l()
            .ok()?;

        env.call_method(
            intent,
            "getParcelableExtra",
            "(Ljava/lang/String;)Landroid/os/Parcelable;",
            &[JValue::Object(&extra_key)],
        )
        .ok()?
        .l()
        .unwrap_or(JObject::null()) // Don't use .ok()? here
    } else {
        tracing::debug!("getting uri via Data");
        env.call_method(intent, "getData", "()Landroid/net/Uri;", &[])
            .ok()? // This catches JNI execution errors
            .l() // This gets the object (even if null)
            .unwrap_or(JObject::null()) // This ensures we keep going even if it's null
    };
    // the URI is null if the OS refuses to open it (SecurityException)
    if uri.is_null() {
        tracing::info!("got null url, cannot handle action");
        return None;
    }

    tracing::debug!("getContentResolver");
    let resolver = env
        .call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .ok()?
        .l()
        .ok()?;

    tracing::debug!("openInputStream");
    let input_stream_result = env.call_method(
        &resolver,
        "openInputStream",
        "(Landroid/net/Uri;)Ljava/io/InputStream;",
        &[JValue::Object(&uri)],
    );

    // CHECK FOR EXCEPTIONS
    // If the user revoked permission, this call throws a SecurityException
    //
    // In JNI, once an exception is thrown in Java, almost every subsequent JNI
    // call will fail/crash until the exception is cleared. If you don't clear
    // it, your app won't just fail to read the file, it will likely lock up or
    // exit entirely the next time you try to do anything with the UI.
    if env.exception_check().ok().unwrap_or(false) {
        env.exception_describe().ok(); // Log the error for your debugging
        env.exception_clear().ok(); // Clear it so the app doesn't crash on the next JNI call
        return None;
    }

    let input_stream = input_stream_result.ok()?.l().ok()?;
    perform_stream_read(env, &input_stream)
}

fn perform_stream_read(env: &mut JNIEnv, input_stream: &JObject) -> Option<String> {
    tracing::info!("perform_stream_read");
    let buffer_size = 1024;
    let byte_array = env.new_byte_array(buffer_size).ok()?;
    let mut result_bytes = Vec::new();

    loop {
        let bytes_read = env
            .call_method(
                input_stream,
                "read",
                "([B)I",
                &[jni::objects::JValue::Object(byte_array.as_ref())],
            )
            .ok()?
            .i()
            .ok()?;

        if bytes_read <= 0 {
            break;
        }

        let mut temp_buf = vec![0i8; bytes_read as usize];
        env.get_byte_array_region(&byte_array, 0, &mut temp_buf[..])
            .ok()?;

        let unsigned_buf: Vec<u8> = temp_buf.into_iter().map(|b| b as u8).collect();
        result_bytes.extend_from_slice(&unsigned_buf);
    }

    // Always close the stream to avoid file descriptor leaks
    let _ = env.call_method(input_stream, "close", "()V", &[]);
    String::from_utf8(result_bytes).ok()
}

/// Get the application's writable directory
pub fn get_app_dir() -> anyhow::Result<PathBuf> {
    use jni::JavaVM;

    let ctx = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }?;
    let mut env = vm.attach_current_thread()?;
    let ctx = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };

    let files_dir = env
        .call_method(ctx, "getFilesDir", "()Ljava/io/File;", &[])?
        .l()?;

    let files_dir_jstring: jni::objects::JString = env
        .call_method(&files_dir, "toString", "()Ljava/lang/String;", &[])?
        .l()?
        .try_into()?;

    let files_dir = env.get_string(&files_dir_jstring)?;
    let files_dir = PathBuf::from(files_dir.to_str()?);

    std::fs::create_dir_all(&files_dir)?;

    Ok(files_dir)
}

pub const SHARE_SHEET_SUPPORTED: bool = true;
/// Share via the OS provided share sheet
pub fn share_contact_file(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    with_android_context(|env, activity| {
        // 1. Validate file exists before even touching JNI
        if !file_path.exists() {
            return None;
        }

        // 2. Load OS Constants and Classes
        let constants = AndroidConstants::fetch(env)?;
        let intent_class = env.find_class("android/content/Intent").ok()?;
        let fp_class = env.find_class("androidx/core/content/FileProvider").ok()?;

        // 3. Create the Java File object
        let file_path_str = file_path.to_string_lossy();
        let j_path = env.new_string(&file_path_str).ok()?;
        let file_obj = env
            .new_object(
                "java/io/File",
                "(Ljava/lang/String;)V",
                &[JValue::Object(&j_path)],
            )
            .ok()?;

        // 4. Get the Content URI via FileProvider
        let authority = format!("{}.fileprovider", get_package_name(env, activity).ok()?);
        let j_authority = env.new_string(&authority).ok()?;

        let uri = env
            .call_static_method(
                fp_class,
                "getUriForFile",
                "(Landroid/content/Context;Ljava/lang/String;Ljava/io/File;)Landroid/net/Uri;",
                &[
                    JValue::Object(activity),
                    JValue::Object(&j_authority),
                    JValue::Object(&file_obj),
                ],
            )
            .ok()?
            .l()
            .ok()?;

        // 5. Build the Intent
        let action_send = env
            .get_static_field(&intent_class, "ACTION_SEND", "Ljava/lang/String;")
            .ok()?
            .l()
            .ok()?;
        let intent = env
            .new_object(
                &intent_class,
                "(Ljava/lang/String;)V",
                &[JValue::Object(&action_send)],
            )
            .ok()?;

        // 6. Set Content and Flags
        let mime_type = env
            .new_string("application/vnd.footnote.contact+json")
            .ok()?;
        env.call_method(
            &intent,
            "setType",
            "(Ljava/lang/String;)Landroid/content/Intent;",
            &[JValue::Object(&mime_type)],
        )
        .ok()?;

        let extra_stream = env
            .get_static_field(&intent_class, "EXTRA_STREAM", "Ljava/lang/String;")
            .ok()?
            .l()
            .ok()?;
        env.call_method(
            &intent,
            "putExtra",
            "(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;",
            &[JValue::Object(&extra_stream), JValue::Object(&uri)],
        )
        .ok()?;

        // Grant read permission to the receiving app
        let flags = constants.flag_read | constants.flag_new_task;
        env.call_method(
            &intent,
            "addFlags",
            "(I)Landroid/content/Intent;",
            &[JValue::Int(flags)],
        )
        .ok()?;

        // 7. Show the System Chooser (The most stable way to trigger a share)
        let title = env.new_string("Share Contact").ok()?;
        let chooser = env
            .call_static_method(
                &intent_class,
                "createChooser",
                "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
                &[JValue::Object(&intent), JValue::Object(&title)],
            )
            .ok()?
            .l()
            .ok()?;

        env.call_method(
            activity,
            "startActivity",
            "(Landroid/content/Intent;)V",
            &[JValue::Object(&chooser)],
        )
        .ok()?;

        Some(())
    })
    .ok_or("Failed to execute Android share")?;

    Ok(())
}

fn get_package_name(
    env: &mut jni::JNIEnv,
    activity: &JObject,
) -> Result<String, Box<dyn std::error::Error>> {
    use jni::objects::JObject;

    let package_name = env
        .call_method(activity, "getPackageName", "()Ljava/lang/String;", &[])?
        .l()?;

    Ok(env.get_string(&package_name.into())?.into())
}
