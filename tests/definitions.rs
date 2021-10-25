use cd::definitions as defs;
use std::convert::TryFrom;

const GET_DATA: &str = include_str!("data/definitions-get.json");
//const SYN_ONLY: &str = include_str!("data/syn-only.json");

#[test]
fn deserialize_get_response() {
    let resp = http::Response::builder()
        .status(200)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(GET_DATA)
        .unwrap();

    let definitions = defs::GetResponse::try_from(resp).unwrap().definitions;

    assert_eq!(definitions.len(), 3);

    {
        let tame_gcs = definitions
            .iter()
            .find(|d| d.coordinates.name == "tame-gcs")
            .unwrap();
        assert!(tame_gcs.described.is_none());
        assert!(tame_gcs.licensed.is_none());
    }

    {
        let syn = definitions
            .iter()
            .find(|d| d.coordinates.name == "syn")
            .unwrap();

        {
            let desc = syn.described.as_ref().unwrap();
            assert_eq!(
                syn.coordinates.revision,
                cd::CoordVersion::Semver(semver::Version::new(1, 0, 14))
            );
            assert_eq!(chrono::NaiveDate::from_ymd(2020, 1, 20), desc.release_date);
            assert_eq!(&defs::SourceLocation {
                r#type: "git".to_owned(),
                provider: "github".to_owned(),
                namespace: "dtolnay".to_owned(),
                name: "syn".to_owned(),
                revision: "855f331cf0e14916a1c3026786b59e6f6b6f2d6f".to_owned(),
                url: "https://github.com/dtolnay/syn/tree/855f331cf0e14916a1c3026786b59e6f6b6f2d6f".to_owned(),
            }, desc.source_location.as_ref().unwrap());
            assert_eq!(
                [
                    ("registry", "https://crates.io/crates/syn"),
                    ("version", "https://crates.io/crates/syn/1.0.14"),
                    (
                        "download",
                        "https://crates.io/api/v1/crates/syn/1.0.14/download"
                    )
                ]
                .iter()
                .map(|(k, v)| (String::from(*k), String::from(*v)))
                .collect::<std::collections::BTreeMap<_, _>>(),
                desc.urls
            );
            assert_eq!(
                defs::Hashes {
                    sha1: "85b0fe2790310f9d6daf04393bc0cf266841d861".to_owned(),
                    sha256: Some(
                        "af6f3550d8dff9ef7dc34d384ac6f107e5d31c8f57d9f28e0081503f547ac8f5"
                            .to_owned()
                    ),
                },
                desc.hashes
            );
            assert_eq!(83, desc.files);
            assert_eq!(
                ["clearlydefined/1.2.0", "licensee/9.13.0", "scancode/3.2.2",]
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>(),
                desc.tools
            );
            assert_eq!(
                defs::Scores {
                    total: 100,
                    date: 30,
                    source: 70,
                },
                desc.tool_score
            );
            assert_eq!(
                defs::Scores {
                    total: 100,
                    date: 30,
                    source: 70,
                },
                desc.score
            );
        }

        {
            let lic = syn.licensed.as_ref().unwrap();
            assert_eq!("Apache-2.0 AND MIT", lic.declared);
            assert_eq!(
                defs::LicenseScore {
                    total: 75,
                    declared: 30,
                    discovered: 0,
                    consistency: 15,
                    spdx: 15,
                    texts: 15,
                },
                lic.tool_score
            );
            assert_eq!(
                defs::LicenseScore {
                    total: 75,
                    declared: 30,
                    discovered: 0,
                    consistency: 15,
                    spdx: 15,
                    texts: 15,
                },
                lic.score
            );

            {
                let core = &lic.facets.core;
                assert_eq!(83, core.attribution.unknown);
                assert_eq!(78, core.discovered.unknown);
                assert_eq!(
                    ["Apache-2.0".to_owned(), "MIT".to_owned()],
                    &core.discovered.expressions[..]
                );
            }
        }

        {
            let files = &syn.files;
            // Normal file
            {
                let build = files.iter().find(|f| f.path == "build.rs").unwrap();
                assert_eq!(
                    Some(defs::Hashes {
                        sha1: "e58729c91f5fa640cdc10944579d803c47071451".to_owned(),
                        sha256: Some(
                            "2570006136c4fed9199b9c23c100a99e1be04d6c6a3e9630a6613a67baedf503"
                                .to_owned()
                        ),
                    }),
                    build.hashes
                );
            }

            // License file
            {
                let ctoml = files.iter().find(|f| f.path == "Cargo.toml").unwrap();
                assert_eq!("MIT", ctoml.license.as_ref().unwrap());
            }
        }
    }

    {
        let tokio = definitions
            .iter()
            .find(|d| d.coordinates.name == "tokio")
            .unwrap();

        let lic = tokio.files.iter().find(|f| f.path == "LICENSE").unwrap();

        assert_eq!("MIT", lic.license.as_ref().unwrap());
        assert_eq!(
            ["Copyright (c) 2019 Tokio".to_owned()],
            &lic.attributions[..]
        );
        assert_eq!(["license".to_owned()], &lic.natures[..]);
    }
}
