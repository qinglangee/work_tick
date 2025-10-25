fn main() {
    slint_build::compile("ui/app-window.slint").expect("Slint build failed");
    let mut res = winres::WindowsResource::new();
    res.set_icon("ui/assets/icon.ico");
    res.compile().expect("Failed to compile resources");
}
