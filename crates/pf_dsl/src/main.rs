use anyhow::{anyhow, Result};
use pf_dsl::traceability::TraceEntity;
use pf_dsl::validator::validate;
use std::collections::BTreeSet;
use std::env;

const DEFAULT_IMPACT_HOPS: usize = 2;

fn usage() -> &'static str {
    "Usage: pf_dsl <file.pf> [--dot | --dot-context | --dot-problem | --dot-decomposition | --report | --gen-rust | --obligations | --alloy | --traceability-md | --traceability-csv] [--impact=requirement:<name>,domain:<name>] [--impact-hops=<n>]"
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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", usage());
        return Ok(());
    }

    let mode = if args.len() > 2 { &args[2] } else { "--dot" };
    let mut impact_seeds = Vec::new();
    let mut impact_hops = DEFAULT_IMPACT_HOPS;

    for arg in args.iter().skip(3) {
        if let Some(raw) = arg.strip_prefix("--impact=") {
            impact_seeds = parse_impact_seeds(raw)?;
        } else if let Some(raw) = arg.strip_prefix("--impact-hops=") {
            impact_hops = raw.parse::<usize>().map_err(|_| {
                anyhow!("invalid value for --impact-hops ('{raw}'), expected non-negative integer")
            })?;
        } else {
            return Err(anyhow!("unknown CLI option '{arg}'. {}", usage()));
        }
    }

    let filename = &args[1];
    let path = std::path::Path::new(filename);

    // Use resolver to handle imports
    match pf_dsl::resolver::resolve(path, None) {
        Ok(problem) => match validate(&problem) {
            Ok(_) => {
                if mode == "--report" {
                    println!("{}", pf_dsl::report_gen::generate_report(&problem));
                } else if mode == "--alloy" {
                    println!("{}", pf_dsl::formal_alloy::generate_alloy(&problem));
                } else if mode == "--obligations" {
                    println!(
                        "{}",
                        pf_dsl::obligations::generate_obligations_markdown(&problem)
                    );
                } else if mode == "--traceability-md" {
                    validate_traceability_seeds(&problem, &impact_seeds)?;
                    println!(
                        "{}",
                        pf_dsl::traceability::generate_traceability_markdown(
                            &problem,
                            &impact_seeds,
                            impact_hops,
                        )
                    );
                } else if mode == "--traceability-csv" {
                    validate_traceability_seeds(&problem, &impact_seeds)?;
                    println!(
                        "{}",
                        pf_dsl::traceability::generate_traceability_csv(
                            &problem,
                            &impact_seeds,
                            impact_hops,
                        )
                    );
                } else if mode == "--dot-context" {
                    println!("{}", pf_dsl::dot_export::to_context_dot(&problem));
                } else if mode == "--dot-problem" {
                    println!("{}", pf_dsl::dot_export::to_problem_dot(&problem));
                } else if mode == "--dot-decomposition" {
                    println!("{}", pf_dsl::dot_export::to_decomposition_dot(&problem));
                } else if mode == "--gen-rust" {
                    match pf_dsl::codegen::generate_rust(&problem) {
                        Ok(code) => println!("{}", code),
                        Err(e) => eprintln!("Error generating code: {}", e),
                    }
                } else {
                    println!("{}", pf_dsl::dot_export::to_dot(&problem));
                }
            }
            Err(errors) => {
                eprintln!("Validation Errors:");
                for err in errors {
                    eprintln!("- {}", err);
                }
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error parsing file: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
