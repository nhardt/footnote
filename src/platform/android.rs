use jni::objects::JObject;
use jni::JNIEnv;
use std::path::PathBuf;

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
