mod cli;
mod error;
mod input;
mod replacer;

use self::input::{App, Source};
use error::{Error, Result};

use clap::Parser;
use replacer::Replacer;

fn main() -> Result<()> {
    let options = cli::Options::parse();

    let source = if !options.globs.is_empty() {
        Source::Files(options.globs)
    } else {
        Source::Stdin
    };

    App::new(
        source,
        Replacer::new(
            options.find,
            options.replace_with,
            options.literal_mode,
            options.flags,
            options.replacements,
        )?,
    )
    .run(options.preview)
}
