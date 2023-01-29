use anyhow::Result;
use vergen::{vergen, Config};

fn main() -> Result<()> {
    // Generate the `cargo:` instructions to fill the appropriate environment variables.
    vergen(Config::default())
}
