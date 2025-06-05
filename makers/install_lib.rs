//! Find this profile's build and install it (optionally install headers if found).
include!(concat!(env!("SCRIPTS"), "/common.rs"));

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO document this in readme or something
    println!(
        "Make sure you have the right permissions (if failed try with `sudo -E`) \
         and the file is built"
    );

    let logging_copy = |from: &str, to: &str, optional: bool| -> std::io::Result<()> {
        if optional && !std::fs::exists(from)? {
            println!(
                "Ignored optional \"{}\": file does not exist (destination: {}).",
                from, to,
            );
        } else {
            print!("Copying \"{}\" -> \"{}\"... ", from, to);
            std::fs::copy(from, to)?;
            println!("Copied.");
        }
        Ok(())
    };

    // LIB
    let lib = format!("lib{}.{}", CRATE_NAME, LIBEXT);
    logging_copy(
        &format!("{}/{}/{}", TARGET, TARGET_PROFILE, lib),
        &format!("{}/{}", INSTALL_FULL_LIBDIR, lib),
        false,
    )?;

    // INCLUDE
    logging_copy(
        C_HEADER,
        &format!("{}/{}.h", INSTALL_FULL_INCLUDEDIR, CRATE_NAME),
        true,
    )?;

    Ok(())
}
