use std::path::Path;

fn main() {
    let dist = Path::new("ui/dist/index.html");
    if !dist.exists() {
        println!("cargo:warning=******************************************************************");
        println!("cargo:warning=*  Angular UI build not found at ui/dist/                         *");
        println!("cargo:warning=*  Run: cd wm-web/ui && npm install && npm run build            *");
        println!("cargo:warning=******************************************************************");
    }
}
