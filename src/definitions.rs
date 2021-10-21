use crate::{ApiResponse, Error};
use bytes::Bytes;
use http::Request;
use serde::Deserialize;
use std::{collections::BTreeMap, convert::TryFrom, fmt};

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
    pub sha1: String,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    pub release_date: chrono::NaiveDate,
    pub source_location: Option<SourceLocation>,
    pub project_website: Option<String>,
    pub urls: BTreeMap<String, String>,
    pub hashes: Hashes,
    pub files: u32,
    pub tools: Vec<String>,
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
    pub unknown: u32,
    #[serde(default)]
    pub parties: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Discovered {
    pub unknown: u32,
    pub expressions: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Facet {
    pub attribution: Attribution,
    pub discovered: Discovered,
    pub files: u32,
}

#[derive(Deserialize, Debug)]
pub struct Facets {
    pub core: Facet,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub declared: String,
    pub facets: Facets,
    pub tool_score: LicenseScore,
    pub score: LicenseScore,
}

#[derive(Deserialize, Debug)]
pub struct File {
    pub path: String,
    pub hashes: Option<Hashes>,
    pub license: Option<String>,
    #[serde(default)]
    pub attributions: Vec<String>,
    #[serde(default)]
    pub natures: Vec<String>,
    pub token: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TopLevelScore {
    pub effective: u8,
    pub tool: u8,
}

#[derive(Debug)]
pub struct Definition {
    pub coordinates: DefCoords,
    /// The description of the component, won't be present if the coordinate
    /// has not been harvested
    pub described: Option<Description>,
    pub licensed: Option<License>,
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
/// the request time is _extremely_ slow and can timeout, so it is recommended
/// you specify a reasonable chunk size and send multiple parallel requests
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
