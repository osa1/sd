use std::fs::File;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;

use crate::{Error, Replacer, Result};

use globwalk::GlobWalkerBuilder;
use walkdir::DirEntry;

#[derive(Debug)]
pub(crate) enum Source {
    Stdin,
    Files(Vec<String>),
}

pub(crate) struct App {
    replacer: Replacer,
    source: Source,
}

impl App {
    pub(crate) fn new(source: Source, replacer: Replacer) -> Self {
        Self { source, replacer }
    }

    pub(crate) fn run(&self, preview: bool) -> Result<()> {
        let is_tty = io::stdout().is_terminal();

        match (&self.source, preview) {
            (Source::Stdin, _) => {
                let mut buffer = Vec::with_capacity(256);
                let stdin = io::stdin();
                let mut handle = stdin.lock();
                handle.read_to_end(&mut buffer)?;

                let stdout = io::stdout();
                let mut handle = stdout.lock();

                if is_tty {
                    handle.write_all(&self.replacer.replace_preview(&buffer))?;
                } else {
                    handle.write_all(&self.replacer.replace(&buffer))?;
                }

                Ok(())
            }

            (Source::Files(globs), false) => {
                use rayon::prelude::*;

                let walker = GlobWalkerBuilder::from_patterns(".", globs)
                    .build()
                    .unwrap();

                let failed_jobs: Vec<(Option<PathBuf>, Error)> = walker
                    .par_bridge()
                    .filter_map(|p| {
                        let p: DirEntry = match p {
                            Ok(p) => p,
                            Err(e) => return Some((None, Error::Walkdir(e))),
                        };
                        match self.replacer.replace_file(&p) {
                            Ok(()) => None,
                            Err(e) => Some((Some(p.into_path()), e)),
                        }
                    })
                    .collect();

                if failed_jobs.is_empty() {
                    Ok(())
                } else {
                    let failed_jobs = crate::error::FailedJobs(failed_jobs);
                    Err(Error::FailedProcessing(failed_jobs))
                }
            }

            (Source::Files(globs), true) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();

                let walker = GlobWalkerBuilder::from_patterns(".", globs)
                    .build()
                    .unwrap();

                for dir_entry in walker {
                    let dir_entry: DirEntry = match dir_entry {
                        Ok(p) => p,
                        Err(_) => break,
                    };
                    let path = dir_entry.path();
                    let file = unsafe { memmap2::Mmap::map(&File::open(path)?)? };
                    if self.replacer.has_matches(&file) {
                        writeln!(handle, "----- FILE {} -----", path.to_string_lossy())?;
                        handle.write_all(&self.replacer.replace_preview(&file))?;
                        writeln!(handle)?;
                    }
                }

                Ok(())
            }
        }
    }
}
