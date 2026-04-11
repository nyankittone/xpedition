use std::path::{Path, PathBuf};
use argh::FromArgs;

#[derive(FromArgs, Debug)]
/// Silly little game made with X11 APIs.
pub struct ProtoArgs {
    /// specify assets directory
    /// (default is {self}/assets, {self} substitutes for the program's running directory)
    #[argh(option, short = 'a', default = "String::from(\"{self}/assets\")")]
    assets_dir: String,
}

#[derive(Debug)]
pub struct Args {
    pub assets_dir: PathBuf,
}

impl Args {
    pub fn new(from: ProtoArgs) -> Result<Self, String> {
        let mut returned = String::new();

        let mut current = from.assets_dir.as_str();
        while let Some((left, right)) = current.split_once("{") {
            returned.push_str(left);

            // get the closing brace
            if let Some((inside, outside)) = right.split_once("}") {
                match inside {
                    "brace" => returned.push_str("{"),
                    "self" => {
                        let thing = std::env::args().nth(0).unwrap();
                        let thing_dir = Path::new(&thing);
                        let thing_dir = thing_dir.parent().unwrap();
                        returned.push_str(thing_dir.to_str().unwrap());
                    },
                    nono => return Err(format!("Invalid substitution inside brace: {nono}")),
                }

                current = outside;
            } else {
                return Err("Forgot closing brace".to_string());
            }
        }

        returned.push_str(current);
        Ok(Self {
            assets_dir: returned.into(),
        })
    }
}

