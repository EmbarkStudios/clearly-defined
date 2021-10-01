use ansi_term::Color;

fn main() -> Result<(), anyhow::Error> {
    let client = cd::client::Client::new();

    let get_reqs =
        cd::definitions::get(std::env::args().skip(1).filter_map(|arg| arg.parse().ok()));

    for get_req in get_reqs {
        let get_res = client.execute::<cd::definitions::GetResponse>(get_req)?;

        for def in get_res.definitions {
            match def.described {
                Some(desc) => {
                    println!("{} - {}", def.coordinates, def.scores.effective);
                    if let Some(lic) = def.licensed {
                        println!(
                            "\tDeclared {} - Discovered {:?}",
                            lic.declared, lic.facets.core.discovered.expressions
                        );
                    }

                    for file in def.files {
                        if let Some(license) = file.license {
                            let color = Color::Green;
                            let color = if file.natures.iter().any(|s| s == "license") {
                                color.bold()
                            } else {
                                color.dimmed()
                            };
                            println!("\tlicense: {} path: {}", color.paint(license), file.path);
                        }
                    }
                }
                None => println!(
                    "{} not harvested",
                    Color::Red.paint(def.coordinates.to_string())
                ),
            }
        }
    }

    Ok(())
}
