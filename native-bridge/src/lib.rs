use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jstring;

pub mod map;

#[no_mangle]
pub extern "system" fn Java_com_unciv_logic_NativeBridge_hello(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let result = std::panic::catch_unwind(|| {
        match env.new_string("Hello from Rust!") {
            Ok(output) => output.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    });
    
    match result {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null_mut()
    }
}

