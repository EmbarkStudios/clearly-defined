use crate::{ApiResponse, Error};
use bytes::Bytes;
use http::Request;
use serde::Deserialize;
use std::{collections::BTreeMap, convert::TryFrom, fmt};

/// The coordinates of a definition
#[derive(Deserialize, Debug)]
pub struct DefCoords {
    #[serde(rename = "type")]
    pub shape: crate::Shape,
    pub provider: crate::Provider,
    pub name: String,
    pub revision: crate::CoordVersion,
}

impl fmt::Display for DefCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.shape.as_str(),
            self.provider.as_str(),
            self.name,
            self.revision,
        )
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Hashes {
    /// The sha-1 hash of a file
    pub sha1: String,
    /// The sha-256 hash of a file
    pub sha256: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Scores {
    pub total: u32,
    pub date: u32,
    pub source: u32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SourceLocation {
    pub r#type: String,
    pub provider: String,
    pub namespace: String,
    pub name: String,
    pub revision: String,
    pub url: String,
}

#[derive(PartialEq, Debug)]
pub struct Date {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

/// Parses a [`Date`] from a string, clearly-defined uses a `YYYY-MM-DD` format
fn date<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Error;

    let date_str: &str = Deserialize::deserialize(deserializer)?;

    let mut iter = date_str.split('-');
    let year = iter
        .next()
        .ok_or_else(|| Error::custom("date doesn't contain a year"))?
        .parse()
        .map_err(Error::custom)?;
    let month = iter
        .next()
        .ok_or_else(|| Error::custom("date doesn't contain a month"))?
        .parse()
        .map_err(Error::custom)?;
    let day = iter
        .next()
        .ok_or_else(|| Error::custom("date doesn't contain a day"))?
        .parse()
        .map_err(Error::custom)?;

    Ok(Date { year, month, day })
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    /// The Datetime when the component was actually released
    #[serde(deserialize_with = "date")]
    pub release_date: Date,
    /// The location where the component was harvested from
    pub source_location: Option<SourceLocation>,
    /// The website associated with the component
    pub project_website: Option<String>,
    /// Urls associated with the component, eg crates.io components will have
    /// the crates.io url, the version specific crates.io url, and the crates.io
    /// download url
    pub urls: BTreeMap<String, String>,
    /// Actually unsure how these hashes are calculated
    pub hashes: Hashes,
    /// The total number of files that were scanned
    pub files: u32,
    /// The tools and curations that were used to harvest the component
    pub tools: Vec<String>,
    /// Scores for the component
    pub tool_score: Scores,
    pub score: Scores,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct LicenseScore {
    pub total: u32,
    pub declared: u32,
    pub discovered: u32,
    pub consistency: u32,
    pub spdx: u32,
    pub texts: u32,
}

#[derive(Deserialize, Debug)]
pub struct Attribution {
    /// The number of files that had no attribution
    pub unknown: u32,
    /// Every attribution that was discovered
    #[serde(default)]
    pub parties: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Discovered {
    /// The number of files that had no, or indeterminant, license information
    pub unknown: u32,
    /// SPDX license expressions that were discovered
    pub expressions: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Facet {
    /// The attributions that were discovered
    pub attribution: Attribution,
    /// The license expressions that were discovered
    pub discovered: Discovered,
    /// The number of files that were crawled
    pub files: u32,
}

#[derive(Deserialize, Debug)]
pub struct Facets {
    /// The only facet I have seen, don't know if there will be more in the future
    pub core: Facet,
}

/// Top-level license information for a definition
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct License {
    /// The license expression that was declared for the component, eg in a
    /// cargo crate this will be the value of the `license` field in the Cargo.toml
    pub declared: String,
    /// Facets of the license
    pub facets: Facets,
    /// Tool scores, they differ from `score`, but don't actually know the
    /// difference in practice
    pub tool_score: LicenseScore,
    pub score: LicenseScore,
}

/// A single file that was crawled when the definition was harvested
#[derive(Deserialize, Debug)]
pub struct File {
    /// The relative path of the file
    pub path: crate::Utf8PathBuf,
    /// The hash information for the file when it was harvested
    pub hashes: Option<Hashes>,
    /// The license that was discovered for the file
    pub license: Option<String>,
    /// Attributions discovered for the file
    #[serde(default)]
    pub attributions: Vec<String>,
    /// "Natures" determined for the file. Unsure how many of them there are
    /// but in practice I have only seen `license` so this should probably be
    /// made into an enum at some point
    #[serde(default)]
    pub natures: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct TopLevelScore {
    pub effective: u8,
    pub tool: u8,
}

#[derive(Debug)]
pub struct Definition {
    /// The specific coordinates the definition pertains to
    pub coordinates: DefCoords,
    /// The description of the component, won't be present if the coordinate
    /// has not been harvested
    pub described: Option<Description>,
    pub licensed: Option<License>,
    /// All of the files that were crawled during the harvest of the component
    pub files: Vec<File>,
    pub scores: TopLevelScore,
}

// Somewhat annoyingly, instead of returning null or some kind of error if a
// coordinate is not in the database, the return will just have a definition
// that is only partially filled out, so we manually deserialize it and just
// set the fields that are meaningless to None even if they have default data
impl<'de> serde::Deserialize<'de> for Definition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de;

        struct DefVisitor;

        impl<'de> de::Visitor<'de> for DefVisitor {
            type Value = Definition;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Definition")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Definition, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut coordinates = None;
                let mut described = None;
                let mut licensed = None;
                let mut files = Vec::new();
                let mut scores = TopLevelScore {
                    effective: 0,
                    tool: 0,
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        "coordinates" => {
                            if coordinates.is_some() {
                                return Err(de::Error::duplicate_field("coordinates"));
                            }

                            coordinates = Some(map.next_value()?);
                        }
                        "described" => {
                            if described.is_some() {
                                return Err(de::Error::duplicate_field("described"));
                            }

                            // Just disregard errors and set it to null
                            let desc: Option<Description> = map.next_value().ok();

                            described = Some(desc);
                        }
                        "licensed" => {
                            if licensed.is_some() {
                                return Err(de::Error::duplicate_field("licensed"));
                            }

                            // Just disregard errors and set it to null
                            let lic: Option<License> = map.next_value().ok();

                            licensed = Some(lic);
                        }
                        "files" => {
                            if !files.is_empty() {
                                return Err(de::Error::duplicate_field("files"));
                            }

                            files = map.next_value()?;
                        }
                        "scores" => {
                            scores = map.next_value()?;
                        }
                        _ => { /* just ignore unknown fields */ }
                    }
                }

                let coordinates =
                    coordinates.ok_or_else(|| de::Error::missing_field("coordinates"))?;
                let described = described.ok_or_else(|| de::Error::missing_field("described"))?;
                let licensed = licensed.ok_or_else(|| de::Error::missing_field("licensed"))?;

                Ok(Definition {
                    coordinates,
                    described,
                    licensed,
                    files,
                    scores,
                })
            }
        }

        const FIELDS: &[&str] = &["coordinates", "described", "licensed", "files", "scores"];
        deserializer.deserialize_struct("Definition", FIELDS, DefVisitor)
    }
}

/// Gets the definitions for the supplied coordinates, note that in addition to
/// this API call being limited to a maximum of 1000 coordinates per request,
/// the request time is sometimes _extremely_ slow and can timeout, so it is
/// recommended you specify a reasonable chunk size and send multiple parallel
/// requests to reduce wall time.
pub fn get<I>(chunk_size: usize, coordinates: I) -> impl Iterator<Item = Request<Bytes>>
where
    I: IntoIterator<Item = crate::Coordinate>,
{
    let chunk_size = std::cmp::min(chunk_size, 1000);
    let mut requests = Vec::new();
    let mut coords = Vec::with_capacity(chunk_size);

    for coord in coordinates {
        coords.push(serde_json::Value::String(format!("{}", coord)));

        if coords.len() == chunk_size {
            requests.push(std::mem::replace(
                &mut coords,
                Vec::with_capacity(chunk_size),
            ));
        }
    }

    if !coords.is_empty() {
        requests.push(coords);
    }

    requests.into_iter().map(|req| {
        let rb = http::Request::builder()
            .method(http::Method::POST)
            .uri(format!("{}/definitions", crate::ROOT_URI))
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(http::header::ACCEPT, "application/json");

        // This..._shouldn't_? fail
        let json = serde_json::to_vec(&serde_json::Value::Array(req))
            .expect("failed to serialize coordinates");

        rb.body(Bytes::from(json)).expect("failed to build request")
    })
}

pub struct GetResponse {
    /// The component definitions, one for each coordinate passed to the get request
    pub definitions: Vec<Definition>,
}

impl ApiResponse<&[u8]> for GetResponse {}
impl ApiResponse<bytes::Bytes> for GetResponse {}

impl<B> TryFrom<http::Response<B>> for GetResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(response: http::Response<B>) -> Result<Self, Self::Error> {
        let (_parts, body) = response.into_parts();

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RawGetResponse {
            #[serde(flatten)]
            items: BTreeMap<String, Definition>,
        }

        let res: RawGetResponse = serde_json::from_slice(body.as_ref())?;

        let mut v = Vec::with_capacity(res.items.len());
        for (_, val) in res.items {
            v.push(val);
        }

        Ok(Self { definitions: v })
    }
}
