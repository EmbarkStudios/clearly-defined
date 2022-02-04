#![doc = include_str!("../README.md")]
// BEGIN - Embark standard lints v5 for Rust 1.55+
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wild_err_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::rc_mutex,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.5 for Rust 1.55+

#[cfg(feature = "client")]
pub mod client;

pub mod definitions;
pub mod error;

pub use error::Error;

use serde::Deserialize;
use std::{convert::TryFrom, fmt, str::FromStr};

pub use camino::Utf8PathBuf;

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

impl<'de> Deserialize<'de> for Shape {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        from_str(deserializer)
    }
}

impl Shape {
    #[inline]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Crate => "crate",
            Self::Git => "git",
        }
    }
}

impl DeFromStr for Shape {}
impl FromStr for Shape {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "crate" => Ok(Shape::Crate),
            "git" => Ok(Shape::Git),
            o => Err(Error::Generic(anyhow::anyhow!("unknown shape '{}'", o))),
        }
    }
}

trait DeFromStr: FromStr<Err = Error> {
    fn des(s: &str) -> Result<Self, Error> {
        Self::from_str(s)
    }
}

#[inline]
fn from_str<'de, T, D>(d: D) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: DeFromStr,
{
    <&'de str>::deserialize(d).and_then(|value| T::des(value).map_err(serde::de::Error::custom))
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Provider {
    /// The canonical crates.io registry for Rust crates
    CratesIo,
    Github,
}

impl Provider {
    #[inline]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CratesIo => "cratesio",
            Self::Github => "github",
        }
    }
}

impl DeFromStr for Provider {}
impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cratesio" => Ok(Provider::CratesIo),
            "github" => Ok(Provider::Github),
            o => Err(Error::Generic(anyhow::anyhow!("unknown provider '{}'", o))),
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

#[derive(Debug, PartialEq)]
pub enum CoordVersion {
    Semver(semver::Version),
    Any(String),
}

impl DeFromStr for CoordVersion {}
impl FromStr for CoordVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Attempt to parse a semver version as that is the most likely
        // version type stored here, at least for Rust
        Ok(match s.parse::<semver::Version>() {
            Ok(vs) => CoordVersion::Semver(vs),
            Err(_err) => CoordVersion::Any(s.to_owned()),
        })
    }
}

impl<'de> serde::Deserialize<'de> for CoordVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        from_str(deserializer)
    }
}

impl fmt::Display for CoordVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Semver(vs) => write!(f, "{}", vs),
            Self::Any(s) => f.write_str(s),
        }
    }
}

/// Defines the coordinates of a specific component
///
/// For example, `crate/cratesio/-/syn/1.0.14`
pub struct Coordinate {
    /// The shape/kind of the component
    pub shape: Shape,
    /// The provider/location of the component
    pub provider: Provider,
    /// Namespace of the component in the provider, or `-` if the provider
    /// does not have namespaces
    pub namespace: Option<String>,
    /// The name of the component
    pub name: String,
    /// The revision of the component, provider dependent
    pub version: CoordVersion,
    /// A curation PR to apply to existing harvested/curated data for the component
    pub curation_pr: Option<u32>,
}

impl std::str::FromStr for Coordinate {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use anyhow::Context as _;

        let mut it = s.split('/');

        let shape = it.next().context("missing shape")?.parse()?;
        let provider = it.next().context("missing provider")?.parse()?;
        let namespace = match it.next().context("missing namespace")? {
            "-" => None,
            other => Some(other.to_owned()),
        };
        let name = it.next().context("missing name")?.to_owned();
        let version = it.next().context("missing version")?.parse()?;

        let curation_pr = match it.next() {
            Some(pr) if pr == "pr" => Some(
                it.next()
                    .context("expected curation PR number")?
                    .parse()
                    .context("unable to parse PR number")?,
            ),
            Some(other) => {
                return Err(Error::Generic(anyhow::anyhow!(
                    "unknown trailing path component '{}'",
                    other
                )));
            }
            None => None,
        };

        Ok(Self {
            shape,
            provider,
            namespace,
            name,
            version,
            curation_pr,
        })
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}/{}",
            self.shape.as_str(),
            self.provider.as_str(),
            self.namespace.as_deref().unwrap_or("-"),
            self.name,
            self.version,
        )?;

        if let Some(pr) = self.curation_pr {
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
