use crate::{frame::Frame, table::EvaluationTable};

use super::domain::Domain;
use super::traits::AIR;
use lambdaworks_math::{
    field::{
        element::FieldElement,
        traits::{IsFFTField, IsField},
    },
    polynomial::Polynomial,
};
use log::{error, info};

/// Validates that the trace is valid with respect to the supplied AIR constraints
pub fn validate_trace<A: AIR>(
    air: &A,
    main_trace_polys: &[Polynomial<FieldElement<A::Field>>],
    aux_trace_polys: &[Polynomial<FieldElement<A::FieldExtension>>],
    domain: &Domain<A::Field>,
    rap_challenges: &A::RAPChallenges,
) -> bool {
    info!("Starting constraints validation over trace...");
    let mut ret = true;

    let main_trace_columns: Vec<_> = main_trace_polys
        .iter()
        .map(|poly| {
            Polynomial::<FieldElement<A::Field>>::evaluate_fft::<A::Field>(
                poly,
                1,
                Some(domain.interpolation_domain_size),
            )
            .unwrap()
        })
        .collect();
    let aux_trace_columns: Vec<_> = aux_trace_polys
        .iter()
        .map(|poly| {
            Polynomial::evaluate_fft::<A::Field>(poly, 1, Some(domain.interpolation_domain_size))
                .unwrap()
        })
        .collect();

    let lde_table =
        EvaluationTable::from_columns(main_trace_columns, aux_trace_columns, A::STEP_SIZE);

    let periodic_columns: Vec<_> = air
        .get_periodic_column_polynomials()
        .iter()
        .map(|poly| {
            Polynomial::<FieldElement<A::Field>>::evaluate_fft::<A::Field>(
                poly,
                1,
                Some(domain.interpolation_domain_size),
            )
            .unwrap()
        })
        .collect();

    // --------- VALIDATE BOUNDARY CONSTRAINTS ------------
    air.boundary_constraints(rap_challenges)
        .constraints
        .iter()
        .for_each(|constraint| {
            let col = constraint.col;
            let step = constraint.step;
            let boundary_value = constraint.value.clone();

            let trace_value = if !constraint.is_aux {
                lde_table.get_main(step, col).clone().to_extension()
            } else {
                lde_table.get_aux(step,  col).clone()
            };

            if boundary_value.clone().to_extension() != trace_value {
                ret = false;
                error!("Boundary constraint inconsistency - Expected value {:?} in step {} and column {}, found: {:?}", boundary_value, step, col, trace_value);
            }
        });

    // --------- VALIDATE TRANSITION CONSTRAINTS -----------
    let n_transition_constraints = air.context().num_transition_constraints();
    let transition_exemptions = &air.context().transition_exemptions;

    let exemption_steps: Vec<usize> = vec![lde_table.n_rows(); n_transition_constraints]
        .iter()
        .zip(transition_exemptions)
        .map(|(trace_steps, exemptions)| trace_steps - exemptions)
        .collect();

    // Iterate over trace and compute transitions
    for step in 0..lde_table.num_steps() {
        let frame =
            Frame::read_from_lde_table(&lde_table, step, 1, &air.context().transition_offsets);

        let periodic_values: Vec<_> = periodic_columns
            .iter()
            .map(|col| col[step].clone())
            .collect();
        let evaluations = air.compute_transition_prover(&frame, &periodic_values, rap_challenges);

        // Iterate over each transition evaluation. When the evaluated step is not from
        // the exemption steps corresponding to the transition, it should have zero as a
        // result
        evaluations.iter().enumerate().for_each(|(i, eval)| {
            // Check that all the transition constraint evaluations of the trace are zero.
            // We don't take into account the transition exemptions.
            if step < exemption_steps[i] && eval != &FieldElement::zero() {
                ret = false;
                error!(
                    "Inconsistent evaluation of transition {} in step {} - expected 0, got {:?}",
                    i, step, eval
                );
            }
        })
    }
    info!("Constraints validation check ended");
    ret
}

pub fn check_boundary_polys_divisibility<F: IsFFTField>(
    boundary_polys: Vec<Polynomial<FieldElement<F>>>,
    boundary_zerofiers: Vec<Polynomial<FieldElement<F>>>,
) {
    for (i, (poly, z)) in boundary_polys
        .iter()
        .zip(boundary_zerofiers.iter())
        .enumerate()
    {
        let (_, b) = poly.clone().long_division_with_remainder(z);
        if b != Polynomial::zero() {
            error!("Boundary poly {} is not divisible by its zerofier", i);
        }
    }
}

/// Validates that the one-dimensional array `data` can be interpreted as two-dimensional
/// array, returning a true when valid and false when not.
pub fn validate_2d_structure<F>(data: &[FieldElement<F>], width: usize) -> bool
where
    F: IsField,
{
    let rows: Vec<Vec<FieldElement<F>>> = data.chunks(width).map(|c| c.to_vec()).collect();
    rows.iter().all(|r| r.len() == rows[0].len())
}
