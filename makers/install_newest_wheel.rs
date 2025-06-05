//! Find the newest wheel in the directory and install it.
include!(concat!(env!("SCRIPTS"), "/common.rs"));

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let targets = env!("CARGO_MAKE_CRATE_TARGET_DIRECTORY");
    let wheels = format!("{}/{}", targets, "wheels");
    let mut pip = std::process::Command::new("pip");
    pip.args(["install", "--force-reinstall"]);

    println!("Make sure a build exists in `{}`", wheels);
    println!(
        "In case this fails to find the right file, manually install with `\"{}\"`",
        command_get_string(&pip).join("\" \"")
    );

    let mut dirs = std::fs::read_dir(&wheels)?
        .filter_map(|i| match i {
            Ok(v) if v.path().extension().map(|i| i.to_str()) == Some(Some("whl")) => Some(v),
            _ => None,
        })
        .collect::<Vec<_>>();
    dirs.sort_by_key(|i| {
        i.metadata()
            .expect("cannot fetch file metadata, manually install the wheel")
            .modified()
            .expect("cannot sort by date, manually install the wheel")
    });
    let last = dirs.last().ok_or(format!(
        "`${}` has no `.whl`, does not exist... did you forget to build the package?",
        wheels
    ))?;
    pip.arg(last.path()).status()?;

    Ok(())
}
