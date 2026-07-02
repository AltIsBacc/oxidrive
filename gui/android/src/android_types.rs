
jni::bind_java_type! { pub Context => "android.context.Context" }
jni::bind_java_type! {
    pub Activity => "android.app.Activity",
    type_map {
        Context => "android.context.Context"
    },
    is_instance_of {
        context: Context
    }
}

jni::bind_java_type! {
    pub Toast => "android.widget.Toast",
    type_map {
        Context => "android.content.Context",
    },
    methods {
        static fn make_text(context: Context, text: JCharSequence, duration: i32) -> Toast,
        fn show(),
    }
}

