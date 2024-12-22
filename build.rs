use std::process::Command;

fn main() {
    let shaders_dir = "src/shaders";
    println!("cargo:rerun-if-changed={shaders_dir}");

    Command::new("slangc")
        .args([
            "src/shaders/main.slang",
            "-target",
            "spirv",
            "-o",
            "assets/shaders/triangle.fragment.spv",
            "-entry",
            "fragmentMain",
        ])
        .status()
        .expect("Failed to run command");
    Command::new("slangc")
        .args([
            "src/shaders/main.slang",
            "-target",
            "spirv",
            "-o",
            "assets/shaders/triangle.vertex.spv",
            "-entry",
            "vertexMain",
        ])
        .status()
        .expect("Failed to run command");
}
