use super::*;

pub fn should_colour(stream: DiagnosticStream, choice: ColourChoice) -> bool {
    match choice {
        ColourChoice::Always => true,
        ColourChoice::Never => false,
        ColourChoice::Auto => {
            if env::var_os("NO_COLOR").is_some() {
                return false;
            }

            if env::var_os("FORCE_COLOR").is_some_and(|value| value.to_string_lossy() != "0")
                || env::var_os("CLICOLOR_FORCE").is_some_and(|value| value.to_string_lossy() != "0")
            {
                return true;
            }

            use std::io::IsTerminal;

            match stream {
                DiagnosticStream::Stdout => io::stdout().is_terminal(),
                DiagnosticStream::Stderr => io::stderr().is_terminal(),
            }
        }
    }
}
