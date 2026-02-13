use crate::ast::*;

fn get_assertion_set<'a>(
    problem: &'a Problem,
    name: &str,
    scope: AssertionScope,
) -> Option<&'a AssertionSet> {
    problem
        .assertion_sets
        .iter()
        .find(|set| set.name == name && set.scope == scope)
}

pub fn generate_obligations_markdown(problem: &Problem) -> String {
    let mut output = String::new();
    output.push_str(&format!("# Proof Obligations: {}\n\n", problem.name));

    if problem.correctness_arguments.is_empty() {
        output.push_str("No correctness arguments were declared.\n");
        return output;
    }

    for argument in &problem.correctness_arguments {
        output.push_str(&format!("## {}\n", argument.name));
        output.push_str(&format!(
            "- Statement: `{} and {} entail {}`\n",
            argument.specification_set, argument.world_set, argument.requirement_set
        ));
        output.push_str(&format!(
            "- Obligation ID: `obl_{}`\n",
            sanitize_obligation_id(&argument.name)
        ));

        if let Some(specification_set) = get_assertion_set(
            problem,
            &argument.specification_set,
            AssertionScope::Specification,
        ) {
            output.push_str(&format!(
                "- Specification set: `{}`\n",
                specification_set.name
            ));
            append_assertions(&mut output, "S", &specification_set.assertions);
        }

        if let Some(world_set) = get_assertion_set(
            problem,
            &argument.world_set,
            AssertionScope::WorldProperties,
        ) {
            output.push_str(&format!("- World set: `{}`\n", world_set.name));
            append_assertions(&mut output, "W", &world_set.assertions);
        }

        if let Some(requirement_set) = get_assertion_set(
            problem,
            &argument.requirement_set,
            AssertionScope::RequirementAssertions,
        ) {
            output.push_str(&format!("- Requirement set: `{}`\n", requirement_set.name));
            append_assertions(&mut output, "R", &requirement_set.assertions);
        }

        output.push('\n');
    }

    output
}

fn append_assertions(output: &mut String, prefix: &str, assertions: &[Assertion]) {
    for (index, assertion) in assertions.iter().enumerate() {
        match &assertion.language {
            Some(language) => output.push_str(&format!(
                "  - {}{}: `{}` (@{})\n",
                prefix,
                index + 1,
                assertion.text,
                language
            )),
            None => output.push_str(&format!(
                "  - {}{}: `{}`\n",
                prefix,
                index + 1,
                assertion.text
            )),
        }
    }
}

fn sanitize_obligation_id(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::generate_obligations_markdown;
    use crate::ast::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    #[test]
    fn renders_obligations_from_correctness_arguments() {
        let problem = Problem {
            name: "P".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![],
            interfaces: vec![],
            requirements: vec![],
            assertion_sets: vec![
                AssertionSet {
                    name: "S_control".to_string(),
                    scope: AssertionScope::Specification,
                    assertions: vec![Assertion {
                        text: "controller updates target".to_string(),
                        language: Some("LTL".to_string()),
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "W_base".to_string(),
                    scope: AssertionScope::WorldProperties,
                    assertions: vec![Assertion {
                        text: "heater responds".to_string(),
                        language: None,
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "R_goal".to_string(),
                    scope: AssertionScope::RequirementAssertions,
                    assertions: vec![Assertion {
                        text: "target achieved".to_string(),
                        language: Some("LTL".to_string()),
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
            ],
            correctness_arguments: vec![CorrectnessArgument {
                name: "A1".to_string(),
                specification_set: "S_control".to_string(),
                world_set: "W_base".to_string(),
                requirement_set: "R_goal".to_string(),
                span: span(),
                source_path: None,
            }],
        };

        let markdown = generate_obligations_markdown(&problem);
        assert!(markdown.contains("obl_A1"));
        assert!(markdown.contains("S_control and W_base entail R_goal"));
        assert!(markdown.contains("controller updates target"));
        assert!(markdown.contains("heater responds"));
        assert!(markdown.contains("target achieved"));
    }
}
