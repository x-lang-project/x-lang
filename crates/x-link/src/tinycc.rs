use crate::LinkOptions;
use x_diagnostics::CompilationFailure;

#[derive(Debug, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    Unsupported(&'static str),
}

#[cfg(all(windows, target_arch = "x86_64"))]
mod windows;

#[cfg(feature = "tinycc-session")]
pub struct Session {
    #[cfg(all(windows, target_arch = "x86_64"))]
    inner: windows::session::Session,
}

#[cfg(feature = "tinycc-session")]
impl Session {
    pub fn new(options: &LinkOptions<'_>) -> Result<Option<Self>, CompilationFailure> {
        #[cfg(all(windows, target_arch = "x86_64"))]
        return windows::session::Session::new(options)
            .map(|session| session.map(|inner| Self { inner }));
        #[cfg(not(all(windows, target_arch = "x86_64")))]
        {
            let _ = options;

            Ok(None)
        }
    }

    pub fn link_elf(
        &mut self,
        objects: &[String],
        output: &str,
        options: &LinkOptions<'_>,
    ) -> Result<LinkOutcome, CompilationFailure> {
        #[cfg(all(windows, target_arch = "x86_64"))]
        return self.inner.link_elf(objects, output, options);
        #[cfg(not(all(windows, target_arch = "x86_64")))]
        {
            let _ = (objects, output, options);

            Ok(LinkOutcome::Unsupported(
                "TinyCC session requires Windows x86-64",
            ))
        }
    }

    pub fn compile_c(
        &mut self,
        source: &str,
        output: &str,
        options: &LinkOptions<'_>,
    ) -> Result<LinkOutcome, CompilationFailure> {
        #[cfg(all(windows, target_arch = "x86_64"))]
        return self.inner.compile_c(source, output, options);
        #[cfg(not(all(windows, target_arch = "x86_64")))]
        {
            let _ = (source, output, options);

            Ok(LinkOutcome::Unsupported(
                "TinyCC session requires Windows x86-64",
            ))
        }
    }
}

pub fn configured(options: &LinkOptions<'_>) -> Result<bool, CompilationFailure> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    return windows::configured(options);
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    {
        let _ = options;

        Ok(false)
    }
}

pub fn link_elf(
    objects: &[String],
    output: &str,
    options: &LinkOptions<'_>,
) -> Result<LinkOutcome, CompilationFailure> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    return windows::link_elf(objects, output, options);
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    {
        let _ = (objects, output, options);

        Ok(LinkOutcome::Unsupported(
            "TinyCC fast linking requires Windows x86-64",
        ))
    }
}

#[cfg(feature = "tinycc-session")]
pub fn compile_c(
    source: &str,
    output: &str,
    options: &LinkOptions<'_>,
) -> Result<LinkOutcome, CompilationFailure> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    return windows::generated::compile_c(source, output, options);
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    {
        let _ = (source, output, options);

        Ok(LinkOutcome::Unsupported(
            "TinyCC generated C requires Windows x86-64",
        ))
    }
}
