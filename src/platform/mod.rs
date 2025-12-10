use std::path::PathBuf;

/// Get the application's writable directory
///
/// On Android: Returns the app's files directory via JNI call to Context.getFilesDir()
/// On other platforms: Returns the user's home directory
#[cfg(target_os = "android")]
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

#[cfg(not(target_os = "android"))]
pub fn get_app_dir() -> anyhow::Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))
}
