use jni::{Env, JavaVM, refs::Cast};
use slint::android::android_activity::AndroidApp;
use std::sync::mpsc;

use crate::android_types::Activity;

pub struct AJNIContext {
    app: AndroidApp,
    jvm: JavaVM,
}

impl AJNIContext {
    pub fn from(app: &AndroidApp) -> Self {
        Self {
            app: app.clone(),
            jvm: unsafe {
                JavaVM::from_raw(app.vm_as_ptr() as _)
            },
        }
    }

    pub fn with_jni<F, T, E>(&self, f: F) -> T
    where
        F: FnOnce(&AndroidApp, &mut Env) -> Result<T, E> + Send + 'static,
        T: Send + 'static,
        E: From<jni::errors::Error> + Send + std::fmt::Debug + 'static,
    {
        let app = self.app.clone();
        let jvm = self.jvm.clone();
        let (tx, rx) = mpsc::channel();

        self.app.run_on_java_main_thread(Box::new(move || {
            let execution_result = jvm
                .attach_current_thread(|env: &mut Env| f(&app, env));

            let _ = tx.send(execution_result);
        }));

        rx.recv()
            .expect("Java main thread dropped the sender or died")
            .expect("JNI operation inside with_jni failed")
    }

    pub fn with_activity<F, T, E>(&self, f: F) -> T
    where
        F: FnOnce(&AndroidApp, &mut Env, &Cast<'_, '_, Activity>) -> Result<T, E> + Send + 'static,
        E: From<jni::errors::Error> + Send + std::fmt::Debug + 'static,
        T: Send + 'static,
    {
        self.with_jni(|app, env| {
            let raw = app.activity_as_ptr() as _;

            let casted = unsafe { 
                env.as_cast_raw::<Activity>(&raw)? 
            };

            f(app, env, &casted)
        })
    }
}

