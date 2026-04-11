use std::path::{Path};
use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum PathOrStr<'a> {
    Path(&'a Path),
    Str(&'a str),
}

impl<'a> From<&'a str> for PathOrStr<'a> {
    fn from(value: &'a str) -> Self {
        PathOrStr::Str(value)
    }
}

impl<'a> From<&'a Path> for PathOrStr<'a> {
    fn from(value: &'a Path) -> Self {
        PathOrStr::Path(value)
    }
}

impl<'a> Display for PathOrStr<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathOrStr::Str(string) => write!(formatter, "{string}"),
            PathOrStr::Path(path) => {
                let printable_path = path.to_string_lossy();
                write!(formatter, "{printable_path}")
            },
        }
    }
}

#[derive(Debug)]
pub struct ResourceError<'a>(PathOrStr<'a>);

impl<'a> From<&'a str> for ResourceError<'a> {
    fn from(value: &'a str) -> Self {
        ResourceError(PathOrStr::Str(value))
    }
}

impl<'a> From<&'a Path> for ResourceError<'a> {
    fn from(value: &'a Path) -> Self {
        ResourceError(PathOrStr::Path(value))
    }
}

impl<'a> Display for ResourceError<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "missing asset: \"{}\"", self.0)
    }
}

impl<'a> std::error::Error for ResourceError<'a> {}

#[derive(Debug)]
pub struct ResourceIntegrityError<'a, 'b> {
    pub broken: PathOrStr<'a>,
    pub missing: ResourceError<'b>,
}

impl<'a, 'b> ResourceIntegrityError<'a, 'b> {
    pub fn associate_str(broken: &'a str, missing: ResourceError<'b>) -> Self {
        Self {
            broken: PathOrStr::Str(broken),
            missing,
        }
    }
    
    pub fn associate_path(broken: &'a Path, missing: ResourceError<'b>) -> Self {
        Self {
            broken: PathOrStr::Path(broken),
            missing,
        }
    }
}

impl<'a, 'b> Display for ResourceIntegrityError<'a, 'b> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "assets \"{}\" not intact: {}", self.broken, self.missing)
    }
}

impl<'a, 'b> Error for ResourceIntegrityError<'a, 'b> {}

