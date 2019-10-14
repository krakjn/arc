use std::collections::BTreeMap as Map;
use std::fmt;
use std::path::PathBuf;


#[derive(Debug)]
pub struct ArcPath {
    pub tld: PathBuf,
    pub arc: PathBuf
}

#[derive(Debug)]
pub struct Repo {
    pub name: String,
    pub deps: Vec<String>
}

pub struct Arc {
    pub path: ArcPath,
    pub remote: Remote,
    pub repos: Vec<Repo>,
    pub arc_cmd: ArcCmd
}
/** 
 * For future maybe https://crates.io/crates/clap
 * Command Line argument parser
 */
#[derive(PartialEq)]
pub enum Cmd {
    Usage,
    Init,
    Checkout,
    Pull,
    Sync,
    Status,
    Reset,
    All,
    Upgrade
}

#[derive(Debug, PartialEq)]
pub enum Opt {
    Verbose(bool),
    Sanitize(bool),
    NewBranch(String),
    Branch(String),
    All(Vec<String>),
    NoOp
}

pub struct ArcCmd {
    pub cmd: Cmd,
    pub opt: Opt,
}

impl ArcCmd {
    pub fn new() -> ArcCmd {
        return ArcCmd {
            cmd: Cmd::Usage,
            opt: Opt::NoOp
        }
    }
}

//--------------------
// J S O N 
//--------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String
}
#[derive(Debug, Deserialize)]
// #[serde(skip_serializing_if = "Map::is_empty")]
// #[serde(untagged)]
pub struct ArcJson {
    pub remote: Remote,
    pub projects: Map<String, Vec<String>>,
}

//--------------------
// C O L O R S 
//--------------------
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum Colors {
// Foreground
    BLACK           = 30,
    RED             = 31,
    GREEN           = 32,
    YELLOW          = 33,
    BLUE            = 34,
    PURPLE          = 35,
    CYAN            = 36,
    WHITE           = 37,
    BRIGHT_BLUE     = 94,
    BRIGHT_PURPLE   = 95,
// Background
    BgBLACK         = 40,
    BgRED           = 41,
    BgGREEN         = 42,
    BgYELLOW        = 43,
    BgBLUE          = 44,
    BgPURPLE        = 45,
    BgCYAN          = 46,
    BgWHITE         = 47,
}

impl fmt::Display for Colors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
            Colors::BLACK          => write!(f, "30"),
            Colors::RED            => write!(f, "31"),
            Colors::GREEN          => write!(f, "32"),
            Colors::YELLOW         => write!(f, "33"),
            Colors::BLUE           => write!(f, "34"),
            Colors::PURPLE         => write!(f, "35"),
            Colors::CYAN           => write!(f, "36"),
            Colors::WHITE          => write!(f, "37"),
            Colors::BRIGHT_BLUE    => write!(f, "94"),
            Colors::BRIGHT_PURPLE  => write!(f, "95"),
            Colors::BgBLACK        => write!(f, "40"),
            Colors::BgRED          => write!(f, "41"),
            Colors::BgGREEN        => write!(f, "42"),
            Colors::BgYELLOW       => write!(f, "43"),
            Colors::BgBLUE         => write!(f, "44"),
            Colors::BgPURPLE       => write!(f, "45"),
            Colors::BgCYAN         => write!(f, "46"),
            Colors::BgWHITE        => write!(f, "47"),
      }
    }
}

#[allow(dead_code)]
pub enum Attrs {
// # Attributes
    NORMAL          = 0,
    BOLD            = 1,
    UNDERLINE       = 4,
    INVERSE         = 7, // foreground becomes background (vice-versa)
}

impl fmt::Display for Attrs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
            Attrs::NORMAL    => write!(f, "0"),
            Attrs::BOLD      => write!(f, "1"),
            Attrs::UNDERLINE => write!(f, "4"),
            Attrs::INVERSE   => write!(f, "7"),
       }
    }
}
