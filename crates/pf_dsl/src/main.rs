use anyhow::{anyhow, Result};
use pf_dsl::traceability::TraceEntity;
use pf_dsl::validator::validate;
use std::collections::BTreeSet;
use std::env;

const DEFAULT_IMPACT_HOPS: usize = 2;

fn usage() -> &'static str {
    "Usage: pf_dsl <file.pf> [--dot | --dot-context | --dot-problem | --dot-decomposition | --report | --gen-rust | --obligations | --alloy | --lean-model | --traceability-md | --traceability-csv | --decomposition-closure | --concern-coverage | --wrspm-report | --wrspm-json | --ddd-pim | --sysml2-text | --sysml2-json | --trace-map-json] [--impact=requirement:<name>,domain:<name>] [--impact-hops=<n>]"
}

fn parse_impact_seeds(raw: &str) -> Result<Vec<TraceEntity>> {
    let mut seeds = BTreeSet::new();
    for token in raw.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }

        if let Some(name) = token.strip_prefix("requirement:") {
            let name = name.trim();
            if name.is_empty() {
                return Err(anyhow!("empty requirement seed in --impact"));
            }
            seeds.insert(TraceEntity::Requirement(name.to_string()));
        } else if let Some(name) = token.strip_prefix("domain:") {
            let name = name.trim();
            if name.is_empty() {
                return Err(anyhow!("empty domain seed in --impact"));
            }
            seeds.insert(TraceEntity::Domain(name.to_string()));
        } else {
            return Err(anyhow!(
                "unsupported impact seed '{token}', expected requirement:<name> or domain:<name>"
            ));
        }
    }

    Ok(seeds.into_iter().collect())
}

fn validate_traceability_seeds(
    problem: &pf_dsl::ast::Problem,
    seeds: &[TraceEntity],
) -> Result<()> {
    let known_requirements: BTreeSet<&str> = problem
        .requirements
        .iter()
        .map(|requirement| requirement.name.as_str())
        .collect();
    let known_domains: BTreeSet<&str> = problem
        .domains
        .iter()
        .map(|domain| domain.name.as_str())
        .collect();

    for seed in seeds {
        match seed {
            TraceEntity::Requirement(name) => {
                if !known_requirements.contains(name.as_str()) {
                    return Err(anyhow!(
                        "unknown requirement impact seed '{name}' for mode traceability"
                    ));
                }
            }
            TraceEntity::Domain(name) => {
                if !known_domains.contains(name.as_str()) {
                    return Err(anyhow!(
                        "unknown domain impact seed '{name}' for mode traceability"
                    ));
                }
            }
            _ => {
                return Err(anyhow!(
                    "impact seeds support only requirement/domain entities"
                ));
            }
        }
    }

    Ok(())
}

fn parse_cli_options(args: &[String]) -> Result<(Vec<TraceEntity>, usize)> {
    let mut impact_seeds = Vec::new();
    let mut impact_hops = DEFAULT_IMPACT_HOPS;
    let mut index = 3;

    while index < args.len() {
        let arg = &args[index];

        if let Some(raw) = arg.strip_prefix("--impact=") {
            impact_seeds = parse_impact_seeds(raw)?;
            index += 1;
            continue;
        }

        if arg == "--impact" {
            if index + 1 >= args.len() {
                return Err(anyhow!("missing value for --impact"));
            }
            impact_seeds = parse_impact_seeds(&args[index + 1])?;
            index += 2;
            continue;
        }

        if let Some(raw) = arg.strip_prefix("--impact-hops=") {
            impact_hops = raw.parse::<usize>().map_err(|_| {
                anyhow!("invalid value for --impact-hops ('{raw}'), expected non-negative integer")
            })?;
            index += 1;
            continue;
        }

        if arg == "--impact-hops" {
            if index + 1 >= args.len() {
                return Err(anyhow!("missing value for --impact-hops"));
            }
            let raw = &args[index + 1];
            impact_hops = raw.parse::<usize>().map_err(|_| {
                anyhow!("invalid value for --impact-hops ('{raw}'), expected non-negative integer")
            })?;
            index += 2;
            continue;
        }

        return Err(anyhow!("unknown CLI option '{arg}'. {}", usage()));
    }

    Ok((impact_seeds, impact_hops))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", usage());
        return Ok(());
    }

    let mode = if args.len() > 2 { &args[2] } else { "--dot" };
    let (impact_seeds, impact_hops) = parse_cli_options(&args)?;

    let filename = &args[1];
    let path = std::path::Path::new(filename);

    match pf_dsl::resolver::resolve(path, None) {
        Ok(problem) => match validate(&problem) {
            Ok(_) => match mode {
                "--report" => {
                    println!("{}", pf_dsl::report_gen::generate_report(&problem));
                }
                "--alloy" => {
                    println!("{}", pf_dsl::formal_alloy::generate_alloy(&problem));
                }
                "--lean-model" => {
                    println!("{}", pf_dsl::lean_export::generate_lean_model(&problem));
                }
                "--obligations" => {
                    println!(
                        "{}",
                        pf_dsl::obligations::generate_obligations_markdown(&problem)
                    );
                }
                "--traceability-md" => {
                    validate_traceability_seeds(&problem, &impact_seeds)?;
                    println!(
                        "{}",
                        pf_dsl::traceability::generate_traceability_markdown(
                            &problem,
                            &impact_seeds,
                            impact_hops,
                        )
                    );
                }
                "--traceability-csv" => {
                    validate_traceability_seeds(&problem, &impact_seeds)?;
                    println!(
                        "{}",
                        pf_dsl::traceability::generate_traceability_csv(
                            &problem,
                            &impact_seeds,
                            impact_hops,
                        )
                    );
                }
                "--dot-context" => {
                    println!("{}", pf_dsl::dot_export::to_context_dot(&problem));
                }
                "--dot-problem" => {
                    println!("{}", pf_dsl::dot_export::to_problem_dot(&problem));
                }
                "--dot-decomposition" => {
                    println!("{}", pf_dsl::dot_export::to_decomposition_dot(&problem));
                }
                "--decomposition-closure" => {
                    println!(
                        "{}",
                        pf_dsl::decomposition_closure::generate_markdown(&problem)
                    );
                }
                "--concern-coverage" => {
                    println!("{}", pf_dsl::concern_coverage::generate_markdown(&problem));
                }
                "--wrspm-report" => {
                    println!("{}", pf_dsl::wrspm::generate_markdown(&problem));
                }
                "--wrspm-json" => match pf_dsl::wrspm::generate_json(&problem) {
                    Ok(json) => println!("{}", json),
                    Err(error) => {
                        eprintln!("Error generating WRSPM JSON: {}", error);
                        std::process::exit(1);
                    }
                },
                "--ddd-pim" => {
                    println!("{}", pf_dsl::pim::generate_ddd_pim_markdown(&problem));
                }
                "--sysml2-text" => {
                    println!("{}", pf_dsl::pim::generate_sysml2_text(&problem));
                }
                "--sysml2-json" => match pf_dsl::pim::generate_sysml2_json(&problem) {
                    Ok(json) => println!("{}", json),
                    Err(error) => {
                        eprintln!("Error generating SysML v2 JSON: {}", error);
                        std::process::exit(1);
                    }
                },
                "--trace-map-json" => match pf_dsl::trace_map::generate_trace_map_json(&problem) {
                    Ok(json) => println!("{}", json),
                    Err(error) => {
                        eprintln!("Error generating trace map JSON: {}", error);
                        std::process::exit(1);
                    }
                },
                "--gen-rust" => match pf_dsl::codegen::generate_rust(&problem) {
                    Ok(code) => println!("{}", code),
                    Err(error) => {
                        eprintln!("Error generating code: {}", error);
                        std::process::exit(1);
                    }
                },
                "--dot" => {
                    println!("{}", pf_dsl::dot_export::to_dot(&problem));
                }
                _ => {
                    return Err(anyhow!("unknown mode '{mode}'. {}", usage()));
                }
            },
            Err(errors) => {
                eprintln!("Validation Errors:");
                for err in errors {
                    eprintln!("- {}", err);
                }
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("Error parsing file: {}", error);
            std::process::exit(1);
        }
    }

    Ok(())
}
