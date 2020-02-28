#[cfg(feature = "client")]
pub mod client;

pub mod api;
pub mod error;

pub use error::Error;

use std::{convert::TryFrom, fmt};

pub const ROOT_URI: &str = "https://api.clearlydefined.io";

// https://api.clearlydefined.io/api-docs/#/definitions/get_definitions
// type/provider/namespace/name/revision
// https://api.clearlydefined.io

/// The "type" of the component
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shape {
    /// A Rust Crate
    Crate,
    Git,
    //Composer,
    //Pod,
    //Maven,
    //Npm,
    //NuGet,
    //PyPi,
    //Gem,
    //SourceArchive,
    //Deb,
    //DebianSources,
}

impl<'de> serde::Deserialize<'de> for Shape {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        from_str(deserializer)
    }
}

impl Shape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Crate => "crate",
            Self::Git => "git",
        }
    }
}

impl DeFromStr for Shape {}
impl std::str::FromStr for Shape {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "crate" => Ok(Shape::Crate),
            "git" => Ok(Shape::Git),
            o => Err(Error::Other(format!("unknown shape '{}'", o))),
        }
    }
}

trait DeFromStr: std::str::FromStr<Err = Error> {
    fn des(s: &str) -> Result<Self, Error> {
        Self::from_str(s)
    }
}

fn from_str<'de, T, D>(d: D) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: DeFromStr,
{
    use serde::de;

    struct Vers<T>(std::marker::PhantomData<T>);

    impl<'de, T> de::Visitor<'de> for Vers<T>
    where
        T: DeFromStr,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            T::des(value).map_err(serde::de::Error::custom)
        }
    }

    d.deserialize_str(Vers(std::marker::PhantomData))
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Provider {
    /// The canonical crates.io registry for Rust crates
    CratesIo,
    Github,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CratesIo => "cratesio",
            Self::Github => "github",
        }
    }
}

impl DeFromStr for Provider {}
impl std::str::FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cratesio" => Ok(Provider::CratesIo),
            "github" => Ok(Provider::Github),
            o => Err(Error::Other(format!("unknown provider '{}'", o))),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Provider {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        from_str(deserializer)
    }
}

#[derive(Debug)]
pub enum CoordVersion {
    Version(semver::Version),
    Any(String),
}

impl<'de> serde::Deserialize<'de> for CoordVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de;

        struct Vers;

        impl<'de> de::Visitor<'de> for Vers {
            type Value = CoordVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("version")
            }

            fn visit_str<E>(self, value: &str) -> Result<CoordVersion, E>
            where
                E: de::Error,
            {
                // Attempt to parse a semver version as that is the most likely
                // version type stored here, at least for Rust
                match value.parse::<semver::Version>() {
                    Ok(vs) => Ok(CoordVersion::Version(vs)),
                    Err(_) => Ok(CoordVersion::Any(value.to_owned())),
                }
            }
        }

        deserializer.deserialize_any(Vers)
    }
}

impl fmt::Display for CoordVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version(vs) => write!(f, "{}", vs),
            Self::Any(s) => f.write_str(&s),
        }
    }
}

/// Defines the coordinates of a specific component
///
/// For example, `https://clearlydefined.io/definitions/crate/cratesio/-/syn/1.0.14`
///
/// shape `crate` – the shape of the component you are looking for. For
/// example, npm, git, nuget, maven, crate...
/// provider `cratesio` – where the component can be found. Examples include
/// npmjs, mavencentral, github, nuget, cratesio...
/// namespace `-` – many component systems have namespaces. GitHub orgs, NPM
/// namespace, Maven group id, … This segment must be supplied. If your
/// component does not have a namespace, use ‘-‘ (ASCII hyphen).
/// name `syn` – the name of the component you want. Given the namespace segment
/// mentioned above, this is just the simple name.
/// revision `1.0.14` – components typically have some differentiator like a
/// version or commit id. Use that here. If this segment is omitted, the latest
/// revision is used (if that makes sense for the provider).
/// pr – literally the string pr. This is a marker segment and must be included
/// if you are looking for the results of applying a particular curation PR to
/// the harvested and curated data for a component
/// number – the GitHub PR number to apply to the existing harvested and curated
/// data.
pub struct Coordinate {
    pub shape: Shape,
    pub provider: Provider,
    pub namespace: Option<String>,
    pub name: String,
    pub version: CoordVersion,
    pub curation_pr: Option<u32>,
}

pub trait Coord: Sized {
    fn shape(&self) -> Shape;
    fn provider(&self) -> Provider;
    fn namespace(&self) -> Option<&str> {
        None
    }
    fn name(&self) -> &str;
    fn version(&self) -> &CoordVersion;
    fn curation_pr(&self) -> Option<u32> {
        None
    }
    fn display(&self) -> CoordDisp<'_, Self>;
}

pub struct CoordDisp<'a, T>(&'a T);

impl<'a, T> fmt::Display for CoordDisp<'a, T>
where
    T: Coord,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}/{}",
            self.0.shape().as_str(),
            self.0.provider().as_str(),
            self.0.namespace().unwrap_or("-"),
            self.0.name(),
            self.0.version()
        )?;

        if let Some(pr) = self.0.curation_pr() {
            write!(f, "/pr/{}", pr)
        } else {
            Ok(())
        }
    }
}

pub trait ApiResponse<B>: Sized + TryFrom<http::Response<B>, Error = Error>
where
    B: AsRef<[u8]>,
{
    fn try_from_parts(resp: http::response::Response<B>) -> Result<Self, Error> {
        if resp.status().is_success() {
            Self::try_from(resp)
        } else {
            // If we get an error, but with a JSON payload, attempt to deserialize
            // an ApiError from it, otherwise fallback to the simple HttpStatus
            // Clearly defined doesn't seem to ever return structured errors?
            // if let Some(ct) = resp
            //     .headers()
            //     .get(http::header::CONTENT_TYPE)
            //     .and_then(|ct| ct.to_str().ok())
            // {
            //     if ct.starts_with("application/json") {
            //         if let Ok(api_err) =
            //             serde_json::from_slice::<error::ApiError>(resp.body().as_ref())
            //         {
            //             return Err(Error::API(api_err));
            //         }
            //     }
            // }

            Err(Error::from(resp.status()))
        }
    }
}
