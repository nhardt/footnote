use jni::objects::JObject;
use jni::objects::JValue;
use jni::JNIEnv;
use jni::{objects::JObject, JNIEnv, JavaVM};
use std::path::PathBuf;
use std::sync::OnceLock;

// We store the JVM globally so any thread can "attach" to it later
static JVM: OnceLock<JavaVM> = OnceLock::new();

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
    use jni::objects::{JObject, JValue};

    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast())? };
    let mut env = vm.attach_current_thread()?;

    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    let file_path_str = file_path.to_string_lossy();
    let j_file_path = env.new_string(&file_path_str)?;

    let file_class = env.find_class("java/io/File")?;
    let file = env.new_object(
        file_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&j_file_path)],
    )?;

    let package_name = get_package_name(&mut env, &activity)?;
    let authority = format!("{}.fileprovider", package_name);
    let j_authority = env.new_string(&authority)?;

    let file_provider_class = env.find_class("androidx/core/content/FileProvider")?;
    let uri = env
        .call_static_method(
            file_provider_class,
            "getUriForFile",
            "(Landroid/content/Context;Ljava/lang/String;Ljava/io/File;)Landroid/net/Uri;",
            &[
                JValue::Object(&activity),
                JValue::Object(&j_authority),
                JValue::Object(&file),
            ],
        )?
        .l()?;

    let intent_class = env.find_class("android/content/Intent")?;
    let action_send = env
        .get_static_field(&intent_class, "ACTION_SEND", "Ljava/lang/String;")?
        .l()?;

    let intent = env.new_object(
        &intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action_send)],
    )?;

    let mime_type = env.new_string("application/vnd.footnote.contact+json")?;
    env.call_method(
        &intent,
        "setType",
        "(Ljava/lang/String;)Landroid/content/Intent;",
        &[JValue::Object(&mime_type)],
    )?;

    let extra_stream = env
        .get_static_field(&intent_class, "EXTRA_STREAM", "Ljava/lang/String;")?
        .l()?;

    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Landroid/os/Parcelable;)Landroid/content/Intent;",
        &[JValue::Object(&extra_stream), JValue::Object(&uri)],
    )?;

    let flag_grant_read = 1;
    env.call_method(
        &intent,
        "addFlags",
        "(I)Landroid/content/Intent;",
        &[JValue::Int(flag_grant_read)],
    )?;

    let chooser_title = env.new_string("Share contact")?;
    let chooser = env
        .call_static_method(
            &intent_class,
            "createChooser",
            "(Landroid/content/Intent;Ljava/lang/CharSequence;)Landroid/content/Intent;",
            &[JValue::Object(&intent), JValue::Object(&chooser_title)],
        )?
        .l()?;

    env.call_method(
        &activity,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&chooser)],
    )?;

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

pub fn handle_incoming_share() -> Option<String> {
    use jni::objects::JObject;

    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()).ok()? };
    let mut env = vm.attach_current_thread().ok()?;
    let activity = unsafe { JObject::from_raw(ctx.context().cast()) };

    let intent = env
        .call_method(&activity, "getIntent", "()Landroid/content/Intent;", &[])
        .ok()?
        .l()
        .ok()?;

    let action = env
        .call_method(&intent, "getAction", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;

    let action_str: String = env.get_string(&action.into()).ok()?.into();

    if action_str != "android.intent.action.VIEW" {
        return None;
    }

    let uri = env
        .call_method(&intent, "getData", "()Landroid/net/Uri;", &[])
        .ok()?
        .l()
        .ok()?;

    read_content_uri(&mut env, &activity, &uri)
}

fn read_content_uri(env: &mut jni::JNIEnv, activity: &JObject, uri: &JObject) -> Option<String> {
    use jni::objects::JValue;

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

    let input_stream = env
        .call_method(
            &resolver,
            "openInputStream",
            "(Landroid/net/Uri;)Ljava/io/InputStream;",
            &[JValue::Object(uri)],
        )
        .ok()?
        .l()
        .ok()?;

    let buffer_size = 1024;
    let byte_array = env.new_byte_array(buffer_size).ok()?;
    let mut result = Vec::new();

    loop {
        let bytes_read = env
            .call_method(
                &input_stream,
                "read",
                "([B)I",
                &[JValue::Object(byte_array.as_ref())],
            )
            .ok()?
            .i()
            .ok()?;

        if bytes_read <= 0 {
            break;
        }

        let mut buf = vec![0i8; bytes_read as usize];
        env.get_byte_array_region(&byte_array, 0, &mut buf[..])
            .ok()?;

        let unsigned_buf: Vec<u8> = buf.into_iter().map(|b| b as u8).collect();
        result.extend_from_slice(&unsigned_buf);
    }

    let _ = env.call_method(&input_stream, "close", "()V", &[]);

    String::from_utf8(result).ok()
}
