mod compile;

fn main() {
    println!("cargo:rerun-if-changed=ui");

    let config = compile::CompilerConfigurationBuilder::default()
        .with_style("material")
        .with_library_paths(vec![
            compile::LibraryPath::new(
                "root", "ui/"
            ),
            compile::LibraryPath::new(
                "material", "ui/libs/material-1.0/material.slint"
            ),
            compile::LibraryPath::new(
                "fragment", "ui/libs/fragment-1.0/fragment.slint"
            ),
            compile::LibraryPath::new(
                "fonts", "ui/fonts"
            ),
        ]);

    compile::compile_with_config(
        "ui/main.slint", config.build()
    ).expect("Failed to compile ui/main.slint");
}

