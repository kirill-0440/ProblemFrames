#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::{parse, parse_error_diagnostic};

    fn token_span(input: &str, token: &str) -> Span {
        let start = input
            .find(token)
            .unwrap_or_else(|| panic!("Expected token '{}'", token));
        Span {
            start,
            end: start + token.len(),
        }
    }

    fn token_span_from(input: &str, start: usize, token: &str) -> Span {
        let local = input[start..].find(token).expect("Expected token");
        let abs = start + local;
        Span {
            start: abs,
            end: abs + token.len(),
        }
    }

    #[test]
    fn test_parse_simple_problem() {
        let input = r#"
            problem: Simple
            domain D kind causal role machine
        "#;
        let problem = parse(input).expect("Failed to parse simple problem");
        assert_eq!(problem.name, "Simple");
        assert_eq!(problem.domains.len(), 1);
        assert_eq!(problem.domains[0].name, "D");
        assert!(matches!(problem.domains[0].kind, DomainKind::Causal));
        assert!(matches!(problem.domains[0].role, DomainRole::Machine));
    }

    #[test]
    fn test_parse_interface() {
        let input = r#"
            problem: I
            interface "A-B" connects A, B {
                shared: {
                    phenomenon e : event [A -> B] controlledBy A
                }
            }
        "#;
        let problem = parse(input).expect("Failed to parse interface");
        assert_eq!(problem.interfaces.len(), 1);
        let iface = &problem.interfaces[0];
        assert_eq!(iface.name, "A-B");
        assert_eq!(iface.connects.len(), 2);
        assert_eq!(iface.shared_phenomena.len(), 1);
        let p = &iface.shared_phenomena[0];
        assert_eq!(p.name, "e");
        assert!(matches!(p.type_, PhenomenonType::Event));
        assert_eq!(p.from.name, "A");
        assert_eq!(p.to.name, "B");
        assert_eq!(p.controlled_by.name, "A");
    }

    #[test]
    fn test_parse_new_types() {
        let input = r#"
            problem: NewTypes
            domain Des kind causal role designed
            interface "Des-Mac" connects Des, Mac {
                shared: {
                    phenomenon Cmd1 : command [Des->Mac] controlledBy Des
                }
            }
        "#;
        let problem = parse(input).expect("Failed to parse");
        assert_eq!(problem.domains[0].kind, DomainKind::Causal);
        assert_eq!(problem.domains[0].role, DomainRole::Designed);
        assert!(matches!(
            problem.interfaces[0].shared_phenomena[0].type_,
            PhenomenonType::Command
        ));
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "prob: BadKeyword"; // typo in keyword
        let result = parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_requirement_missing_frame() {
        let input = r#"
            problem: MissingFrame
            domain M kind causal role machine

            requirement "R1" {
                constrains: M
            }
        "#;

        assert!(
            parse(input).is_err(),
            "requirement without frame must be rejected at parse time"
        );
    }

    #[test]
    fn test_parse_requirement_duplicate_fields() {
        let input = r#"
            problem: DuplicateFields
            domain M kind causal role machine

            requirement "R1" {
                frame: RequiredBehavior
                frame: RequiredBehavior
                constrains: M
            }
        "#;

        assert!(parse(input).is_err());
    }

    #[test]
    fn test_parse_requirement_custom_frame_as_string() {
        let input = r#"
            problem: CustomFrame
            domain M kind causal role machine

            requirement "R1" {
                frame: "ProblemFrames Custom"
                constrains: M
            }
        "#;

        let problem = parse(input).expect("Failed to parse custom frame");
        assert_eq!(problem.requirements.len(), 1);
        assert!(matches!(
            &problem.requirements[0].frame,
            crate::ast::FrameType::Custom(name) if name == "ProblemFrames Custom"
        ));
    }

    #[test]
    fn test_parse_requirement_empty_string_frame() {
        let input = r#"
            problem: EmptyFrame
            domain M kind causal role machine

            requirement "R1" {
                frame: ""
                constrains: M
            }
        "#;

        assert!(parse(input).is_err());
    }

    #[test]
    fn test_parse_error_diagnostic_for_missing_frame() {
        let input = r#"
            problem: MissingFrameDiag
            domain M kind causal role machine
            requirement "R1" {
                constrains: M
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "constrains: M");

        assert!(message.contains("missing required field 'frame:'"));
        assert_eq!(span, expected);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_invalid_domain_kind() {
        let input = r#"
            problem: InvalidDomainKind
            domain M kind machine role given
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "machine");

        assert!(message.contains("expected"));
        assert!(span.start >= expected.start);
        assert!(span.start < expected.end);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_invalid_domain_role() {
        let input = r#"
            problem: InvalidDomainRole
            domain M kind causal role human
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "human");

        assert!(message.contains("expected"));
        assert!(span.start >= expected.start);
        assert!(span.start < expected.end);
    }

    #[test]
    fn test_parse_error_diagnostic_for_invalid_domain_name_identifier() {
        let input = r#"
            problem: InvalidDomainName
            domain "M" kind causal role machine
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "\"M\"");

        assert!(!message.is_empty());
        assert!(span.start <= expected.start);
        assert!(span.end >= expected.start + 1);
    }

    #[test]
    fn test_parse_error_diagnostic_for_invalid_problem_declaration() {
        let input = r#"
            problem InvalidProblem
            domain M kind causal role machine
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "problem");

        assert!(!message.is_empty());
        assert!(span.start >= expected.start);
        assert!(span.start < expected.end);
    }

    #[test]
    fn test_parse_error_diagnostic_for_empty_input() {
        let input = "";

        assert!(parse_error_diagnostic(input).is_none());
    }

    #[test]
    fn test_parse_error_diagnostic_for_missing_required_problem_name() {
        let input = r#"
            problem:
            domain M kind causal role machine
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "problem:");

        assert!(!message.is_empty());
        assert!(span.start >= expected.start);
        assert!(span.start < input.len());
    }

    #[test]
    fn test_parse_error_diagnostic_returns_none_for_valid_input() {
        let input = r#"
            problem: Valid
            domain M kind causal role machine
        "#;

        assert!(parse_error_diagnostic(input).is_none());
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_invalid_import_decl() {
        let input = r#"
            problem: InvalidImport
            import file.pf
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "file.pf");

        assert!(message.contains("string") || message.contains("\""));
        assert!(span.start >= expected.start);
        assert!(span.start < expected.end);
    }

    #[test]
    fn test_parse_error_diagnostic_for_malformed_interface_connects_list() {
        let input = r#"
            problem: MalformedConnects
            domain M kind causal role machine
            interface "A-B" connects A, {
                shared: {
                }
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, ",");

        assert!(!message.is_empty());
        assert!(span.start >= expected.start);
        assert!(span.start < expected.end + 4);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_invalid_correctness_argument() {
        let input = r#"
            problem: InvalidCorrectness
            domain M kind causal role machine
            correctnessArgument A1 {
                prove S_control W_base entail R_goal
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "prove");

        assert!(message.contains("expected"));
        assert!(span.start >= expected.start);
        assert!(span.start < expected.end + 8);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_invalid_interface_decl() {
        let input = r#"
            problem: InvalidInterface
            interface "A-B" connects A, B {
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "}");

        assert!(message.contains("shared") || message.contains("shared:"));
        assert_eq!(span.start, expected.start);
        assert_eq!(span.end, expected.end);
    }

    #[test]
    fn test_parse_error_diagnostic_for_invalid_requirement_name() {
        let input = r#"
            problem: InvalidRequirementName
            domain M kind causal role machine
            requirement R1 {
                frame: RequiredBehavior
                constrains: M
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "R1");

        assert!(!message.is_empty());
        assert_eq!(span.start, expected.start);
        assert_eq!(span.end, expected.start + 1);
    }

    #[test]
    fn test_parse_error_diagnostic_for_malformed_interface_name() {
        let input = r#"
            problem: MalformedInterface
            domain M kind causal role machine
            interface "A-B {
                shared: {
                }
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "\"A-B {");

        assert!(!message.is_empty());
        assert!(span.start >= expected.start);
        assert!(span.end == span.start + 1);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_duplicate_requirement_fields() {
        let input = r#"
            problem: DuplicateFieldsDiag
            domain M kind causal role machine
            requirement "R1" {
                frame: RequiredBehavior
                frame: RequiredBehavior
                constrains: M
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let first = input
            .find("frame: RequiredBehavior")
            .expect("Expected first frame field");
        let second = input[first + 1..]
            .find("frame: RequiredBehavior")
            .map(|idx| idx + first + 1)
            .expect("Expected duplicate frame field");
        let expected = token_span_from(input, second, "frame: RequiredBehavior");

        assert!(message.contains("has duplicate field 'frame'"));
        assert_eq!(span, expected);
    }

    #[test]
    fn test_parse_error_diagnostic_for_subproblem_missing_fields() {
        let input = r#"
            problem: SubproblemMissingDiag
            domain M kind causal role machine
            domain A kind causal role given
            requirement "R1" {
                frame: RequiredBehavior
                constrains: A
            }
            subproblem Core {
                participants: M, A
                requirements: "R1"
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "subproblem Core");

        assert!(message.contains("missing required field 'machine:'"));
        assert!(span.start >= expected.start);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_duplicate_subproblem_participants() {
        let input = r#"
            problem: SubproblemDuplicateParticipantDiag
            domain M kind causal role machine
            domain A kind causal role given
            requirement "R1" {
                frame: RequiredBehavior
                constrains: A
            }
            subproblem Core {
                machine: M
                participants: M, A, M
                requirements: "R1"
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span_from(
            input,
            input
                .rfind("participants: M, A, ")
                .expect("Expected participants list")
                .saturating_add("participants: M, A, ".len()),
            "M",
        );

        assert!(message.contains("has duplicate participant"));
        assert_eq!(span, expected);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_duplicate_subproblem_requirement_refs() {
        let input = r#"
            problem: SubproblemDuplicateReqRefDiag
            domain M kind causal role machine
            domain A kind causal role given
            requirement "R1" {
                frame: RequiredBehavior
                constrains: A
            }
            subproblem Core {
                machine: M
                participants: M, A
                requirements: "R1", "R1"
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let duplicate = input
            .rfind("\"R1\"")
            .expect("Expected duplicate requirement reference");
        let expected = Span {
            start: duplicate,
            end: duplicate + 4,
        };

        assert!(message.contains("has duplicate requirement reference"));
        assert_eq!(span, expected);
    }

    #[test]
    fn test_parse_error_diagnostic_has_span_for_unknown_top_level_token() {
        let input = r#"
            problem: ExtraTopLevel
            domain M kind causal role machine
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "}");

        assert!(!message.is_empty());
        assert_eq!(span, expected);
    }

    #[test]
    fn test_parse_error_diagnostic_for_invalid_phenomenon_arrow_syntax() {
        let input = r#"
            problem: BadPhenomenon
            domain M kind causal role machine
            domain N kind causal role given
            interface "M-N" connects M, N {
                shared: {
                    phenomenon e : event [M - N] controlledBy M
                }
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, "phenomenon e");

        assert!(!message.is_empty());
        assert!(span.start <= expected.end + 12);
        assert!(span.end >= expected.start);
    }

    #[test]
    fn test_parse_error_diagnostic_for_subproblem_requirements_invalid_token() {
        let input = r#"
            problem: InvalidSubproblemRequirements
            domain M kind causal role machine
            domain A kind causal role given
            requirement "R1" {
                frame: RequiredBehavior
                constrains: A
            }
            subproblem Core {
                machine: M
                participants: M, A
                requirements: "R1", A
            }
        "#;

        let result = parse_error_diagnostic(input).expect("Expected diagnostic");
        let (span, message) = result;
        let expected = token_span(input, " A");

        assert!(!message.is_empty());
        assert!(span.start >= expected.start);
    }

    #[test]
    fn test_parse_assertion_sets_and_correctness_argument() {
        let input = r#"
            problem: Formalized
            worldProperties W_base {
                assert "physics is stable" @LTL
            }
            specification S_control {
                assert "commands eventually applied" @LTL
            }
            requirementAssertions R_goal {
                assert "target eventually met" @LTL
            }
            correctnessArgument A1 {
                prove S_control and W_base entail R_goal
            }
        "#;

        let problem = parse(input).expect("Failed to parse formal blocks");
        assert_eq!(problem.assertion_sets.len(), 3);
        assert_eq!(problem.correctness_arguments.len(), 1);

        let worlds = problem
            .assertion_sets
            .iter()
            .filter(|set| matches!(set.scope, AssertionScope::WorldProperties))
            .count();
        assert_eq!(worlds, 1);
        assert_eq!(
            problem.correctness_arguments[0].specification_set,
            "S_control"
        );
        assert_eq!(problem.correctness_arguments[0].world_set, "W_base");
        assert_eq!(problem.correctness_arguments[0].requirement_set, "R_goal");
    }

    #[test]
    fn test_parse_subproblem_decomposition() {
        let input = r#"
            problem: Decomposed
            domain M kind causal role machine
            domain A kind causal role given

            requirement "R1" {
                frame: RequiredBehavior
                constrains: A
            }

            subproblem CoreControl {
                machine: M
                participants: M, A
                requirements: "R1"
            }
        "#;

        let problem = parse(input).expect("Failed to parse subproblem");
        assert_eq!(problem.subproblems.len(), 1);
        let subproblem = &problem.subproblems[0];
        assert_eq!(subproblem.name, "CoreControl");
        assert_eq!(
            subproblem.machine.as_ref().map(|r| r.name.as_str()),
            Some("M")
        );
        assert_eq!(subproblem.participants.len(), 2);
        assert_eq!(subproblem.requirements.len(), 1);
        assert_eq!(subproblem.requirements[0].name, "R1");
    }

    #[test]
    fn test_parse_subproblem_missing_required_fields() {
        let input = r#"
            problem: SubproblemMissing
            domain M kind causal role machine

            subproblem Core {
                participants: M
                requirements: "R1"
            }
        "#;

        assert!(parse(input).is_err());
    }

    #[test]
    fn test_parse_subproblem_duplicate_fields() {
        let input = r#"
            problem: SubproblemDuplicate
            domain M kind causal role machine
            domain A kind causal role given
            requirement "R1" {
                frame: RequiredBehavior
                constrains: A
            }

            subproblem Core {
                machine: M
                participants: M, A
                participants: M, A
                requirements: "R1"
            }
        "#;

        assert!(parse(input).is_err());
    }

    #[test]
    fn test_parse_subproblem_duplicate_requirement_refs() {
        let input = r#"
            problem: SubproblemDupReq
            domain M kind causal role machine
            domain A kind causal role given
            requirement "R1" {
                frame: RequiredBehavior
                constrains: A
            }

            subproblem Core {
                machine: M
                participants: M, A
                requirements: "R1", "R1"
            }
        "#;

        assert!(parse(input).is_err());
    }
}
