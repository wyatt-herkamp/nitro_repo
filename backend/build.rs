use vergen::EmitBuilder;

fn main() {
    if let Err(error) = EmitBuilder::builder()
        .all_build()
        .all_git()
        .all_cargo()
        .all_rustc()
        .all_sysinfo()
        .emit()
    {
        println!("Error: {error:?}");
    }
}
