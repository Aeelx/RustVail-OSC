extern crate winresource;

fn main() {
    // Add icon to the executable when building for windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/RVO.ico");
        res.compile().unwrap();
    }
}
