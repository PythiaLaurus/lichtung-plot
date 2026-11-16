use std::sync::{Arc, Mutex};

use crate::numeric::source::NumericBinding;
use crate::{
    NumericFormat, NumericIdentity, NumericLabels, NumericLocale, NumericOperand, NumericPrecision,
    NumericPurpose, NumericRole,
};

fn with_source<T>(visit: impl FnOnce(NumericBinding<'_>) -> T) -> T {
    let plot = crate::Plot::scatter(
        crate::NumericValues::new("x", [0.0, 1.0]),
        crate::NumericValues::new("y", [0.0, 1.0]),
    )
    .erase();
    let operation = crate::font::canonical::OperationContext::new();
    let scope = crate::orchestrator::registration::Scope::new(
        plot.description(),
        &operation,
        0,
        crate::layout::ComponentAddress::root()
            .child(crate::layout::AddressRole::PlotBody, 0)
            .unwrap(),
        crate::orchestrator::registration::RowSelection::Full,
        None,
        None,
    )
    .unwrap();
    let mut registry =
        crate::orchestrator::registration::PlotRegistryBuilder::new(operation.root_identity());
    let id = registry
        .reserve(
            plot.description(),
            scope.address,
            scope.numeric_owner,
            scope.axis_numeric_owner,
            scope.rows,
            scope.render_name,
            crate::orchestrator::registration::PlotScopePolicy::for_source(
                plot.description(),
                None,
            ),
            plot.description().resolve_theme().unwrap(),
            &operation,
        )
        .unwrap();
    let panel = &registry.panels[id.slot];
    visit(
        panel
            .scope
            .numeric_binding(panel.source, &panel.row_source, &operation)
            .unwrap(),
    )
}

fn info(
    source: &NumericBinding<'_>,
    values: &[f64],
    purpose: NumericPurpose,
) -> crate::numeric::context::ContextInfo {
    source
        .indexed(
            &values
                .iter()
                .map(|value| crate::numeric::operand::Operand::number(*value, false))
                .collect::<Vec<_>>(),
            NumericRole::XAxis,
            purpose,
            Some(crate::numeric::context::DomainInfo::axis(
                (0.0, 10.0),
                crate::AxisScale::Linear,
                crate::layout::RangeProvenance::Configured,
                Some((0.0, 10.0)),
            )),
        )
        .unwrap()
}

#[test]
fn value_functions_keep_ordered_occurrences_whole_set_helpers_and_operation_identity() {
    with_source(|source| {
        let base = NumericFormat::default()
            .precision(NumericPrecision::AdaptiveFractional { min: 0, max: 2 })
            .unwrap()
            .locale(NumericLocale::named("ar-001").unwrap())
            .unwrap()
            .prefix("ID012 ")
            .unwrap();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let observed = calls.clone();
        let format = base.clone().value_function(move |context| {
            assert_eq!(context.set().values().len(), 3);
            assert_eq!(
                context.set().domain().unwrap().data_limits(),
                Some((0.0, 10.0))
            );
            let NumericOperand::Binary64(value) = context.value().operand() else {
                panic!("ordinary source")
            };
            match context.value().identity() {
                NumericIdentity::Tick { source_index, .. }
                | NumericIdentity::Probe {
                    index: source_index,
                } => assert_eq!(source_index, context.index()),
                _ => panic!("ordered numeric identity"),
            }
            observed.lock().unwrap().push((
                context.set().purpose(),
                context.index(),
                value.to_bits(),
            ));
            Ok(format!("{}:{}", context.index(), context.format_builtin()?))
        });
        let context = info(&source, &[1.01, 1.02, 1.01], NumericPurpose::ActualSet);
        let set = format
            .materialize(source.operation, context.clone(), true)
            .unwrap()
            .unwrap();
        assert_eq!(set.labels, ["0:ID012 ١٫٠١", "1:ID012 ١٫٠٢", "2:ID012 ١٫٠١"]);
        assert_eq!(set.precision, 2);
        assert_eq!(
            set.members,
            [1.01, 1.02, 1.01].map(|value| crate::numeric::operand::Operand::number(value, false))
        );
        assert_eq!(
            set,
            format
                .materialize(source.operation, context.clone(), true)
                .unwrap()
                .unwrap()
        );
        assert_eq!(calls.lock().unwrap().len(), 3);
        let mut other_owner = source.owner.clone();
        other_owner.canvas = Some((1, 2));
        let other = NumericBinding {
            owner: &other_owner,
            ..source
        };
        format
            .materialize(
                source.operation,
                info(&other, &[1.01, 1.02, 1.01], NumericPurpose::ActualSet),
                true,
            )
            .unwrap();
        format
            .materialize(
                source.operation,
                info(&source, &[1.01, 1.02, 1.01], NumericPurpose::IntrinsicProbe),
                true,
            )
            .unwrap();
        assert_eq!(calls.lock().unwrap().len(), 9);
        assert_eq!(format.clone(), format);
        assert_eq!(format.clone().builtin(), base);
        #[expect(
            clippy::redundant_closure_for_method_calls,
            reason = "the method item fixes the view lifetime instead of satisfying the higher-ranked callback"
        )]
        let independent = base.value_function(|context| context.format_builtin());
        assert_ne!(format, independent);
        source.operation.seal_fonts().unwrap();
        assert!(format.materialize(source.operation, context, true).is_err());
        assert_eq!(calls.lock().unwrap().len(), 9);
    });
}

#[test]
fn callback_cardinality_controls_errors_and_empty_owners_precede_font_work() {
    with_source(|source| {
        let context = info(&source, &[1.0, 2.0], NumericPurpose::ActualSet);
        let short =
            NumericFormat::default().set_function(|_| Ok(NumericLabels::new(vec!["one".into()])));
        assert!(matches!(
            short.materialize(source.operation, context.clone(), true),
            Err(crate::Error::Data(_))
        ));
        let invalid = NumericFormat::default().value_function(|_| Ok("tab\ttext".into()));
        assert!(invalid
            .materialize(source.operation, context.clone(), true)
            .is_err());
        let failed = NumericFormat::default()
            .set_function(|_| Err(crate::Error::Data("application refusal".into())));
        let error = failed
            .materialize(source.operation, context.clone(), true)
            .unwrap_err()
            .to_string();
        assert!(error.contains("XAxis") && error.contains("application refusal"));
        let collision = NumericFormat::default().value_function(|_| Ok("same".into()));
        assert!(collision
            .materialize(source.operation, context, true)
            .unwrap()
            .is_none());
        let empty = failed
            .materialize(
                source.operation,
                info(&source, &[], NumericPurpose::ActualSet),
                true,
            )
            .unwrap()
            .unwrap();
        assert!(empty.labels.is_empty() && empty.explanation.is_none());
    });
}

#[test]
fn ignored_helpers_never_allocate_fields_and_used_helpers_keep_exposed_factors() {
    with_source(|source| {
        let enormous = NumericFormat::default()
            .field(Some(
                crate::NumericField::new(usize::MAX, ' ', crate::NumericAlignment::Right).unwrap(),
            ))
            .unwrap()
            .value_function(|context| Ok(context.index().to_string()));
        assert_eq!(
            enormous
                .materialize(
                    source.operation,
                    info(&source, &[1.0, 2.0], NumericPurpose::ActualSet),
                    true
                )
                .unwrap()
                .unwrap()
                .labels,
            ["0", "1"]
        );
        #[expect(
            clippy::redundant_closure_for_method_calls,
            reason = "the method item does not satisfy the higher-ranked context lifetime"
        )]
        let with_helper = enormous.value_function(|context| context.format_builtin());
        assert!(with_helper
            .materialize(
                source.operation,
                info(&source, &[1.0], NumericPurpose::ActualSet),
                true
            )
            .is_err());
        let frozen = NumericFormat::default()
            .precision(NumericPrecision::AdaptiveFractional { min: 0, max: 4 })
            .unwrap()
            .prefixes(
                crate::NumericPrefixes::si(
                    crate::NumericSelection::WholeSet,
                    crate::NumericPromotion::PromoteOnRounding,
                )
                .unwrap(),
            )
            .unwrap()
            .set_function(|context| {
                let factors = context.common_factors().unwrap();
                assert_eq!(
                    factors.divisor(),
                    &crate::NumericFactor::PowerOfTen("3".into())
                );
                let labels = context.format_builtin()?;
                assert_eq!(labels.labels(), ["0.999k", "1.000k"]);
                assert_eq!(
                    context.common_factors().unwrap().divisor(),
                    factors.divisor()
                );
                Ok(labels)
            });
        let set = frozen
            .materialize(
                source.operation,
                info(&source, &[999.25, 999.5], NumericPurpose::ActualSet),
                true,
            )
            .unwrap()
            .unwrap();
        assert_eq!(set.precision, 3);
    });
}
