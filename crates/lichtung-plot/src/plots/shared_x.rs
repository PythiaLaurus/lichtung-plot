//! Font-independent scientific domains and categorical axis sources.

mod prepared;
pub(crate) use prepared::{
    prepare_line_plot, prepare_pair_axes, prepare_plot, prepare_plot_registration,
    prepare_reference_plot, prepare_scatter_plot,
};

#[cfg(test)]
thread_local! {
    pub(crate) static AXIS_PREPARATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
