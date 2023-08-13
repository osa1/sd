use std::fs::File;
use std::io::{self, IsTerminal, Read, Write};
use std::path::PathBuf;

use crate::error::{Error, FailedJobs};
use crate::{Replacer, Result};

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

                let mut errors: Vec<Error> = Vec::new();
                let mut paths: Vec<PathBuf> = Vec::new();
                for glob in globs {
                    collect_glob_paths(glob, &mut paths, &mut errors);
                }

                let replace_errors: Vec<(Option<PathBuf>, Error)> = paths
                    .par_iter()
                    .filter_map(|p| {
                        if let Err(e) = self.replacer.replace_file(p) {
                            Some((Some(p.to_owned()), e))
                        } else {
                            None
                        }
                    })
                    .collect();

                let failed_jobs: Vec<(Option<PathBuf>, Error)> = errors
                    .into_iter()
                    .map(|e| (None, e))
                    .chain(replace_errors.into_iter())
                    .collect();

                if failed_jobs.is_empty() {
                    Ok(())
                } else {
                    Err(Error::FailedProcessing(FailedJobs(failed_jobs)))
                }
            }

            (Source::Files(globs), true) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();

                let mut errors: Vec<Error> = Vec::new();
                let mut paths: Vec<PathBuf> = Vec::new();
                for glob in globs {
                    collect_glob_paths(glob, &mut paths, &mut errors);
                }

                for error in errors {
                    writeln!(handle, "{}", error)?;
                }

                for path in paths {
                    let file = unsafe { memmap2::Mmap::map(&File::open(&path)?)? };
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

fn collect_glob_paths(glob: &str, paths: &mut Vec<PathBuf>, errors: &mut Vec<Error>) {
    let glob_paths = match glob::glob(glob) {
        Ok(paths) => paths,
        Err(err) => return errors.push(Error::from(err)),
    };
    for path in glob_paths {
        match path {
            Ok(path) => paths.push(path),
            Err(err) => errors.push(Error::from(err)),
        }
    }
}
