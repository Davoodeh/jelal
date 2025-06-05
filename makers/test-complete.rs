//! Run `doc`, `clippy` and `test` for all possible feature flag combinations.
include!(concat!(env!("SCRIPTS"), "/common.rs"));

fn main() -> Result<(), ()> {
    cargo_verb_all_feature_combinations_run("clippy")?;
    cargo_verb_all_feature_combinations_run("doc")?;
    cargo_verb_all_feature_combinations_run("test")?;
    Ok(())
}
