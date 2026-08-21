use super::Chart;
use crate::spec::PlotKind;

impl Chart<'_> {
    /// Select a canonical plot kind for rendering.
    pub fn select(&mut self, kind: PlotKind) -> &mut Self {
        self.plot_kind = Some(kind);
        self
    }

    /// Terminate the builder: select the scatter plot. Returns
    /// `&mut Self` so the caller can then call
    /// `render_to_svg` / `render_to_png` on the same `Chart`
    /// per `02_CRATES.md` §2.4.
    pub fn scatter(&mut self) -> &mut Self {
        self.select(PlotKind::Scatter)
    }

    /// Terminate the builder: select a histogram.
    pub fn histogram(&mut self) -> &mut Self {
        self.select(PlotKind::Histogram)
    }

    /// Terminate the builder: select a Gaussian density plot.
    pub fn density(&mut self) -> &mut Self {
        self.select(PlotKind::Density)
    }

    /// Terminate the builder: select a normal Q-Q plot.
    pub fn qq(&mut self) -> &mut Self {
        self.select(PlotKind::Qq)
    }

    /// Terminate the builder: select a point-interval plot.
    pub fn point_interval(&mut self) -> &mut Self {
        self.select(PlotKind::PointInterval)
    }

    /// Terminate the builder: select an ordered line plot.
    pub fn line(&mut self) -> &mut Self {
        self.select(PlotKind::Line)
    }

    /// Terminate the builder: select an interaction-profile plot.
    pub fn interaction(&mut self) -> &mut Self {
        self.select(PlotKind::Interaction)
    }

    /// Terminate the builder: select a model-coefficient plot.
    pub fn coefficient(&mut self) -> &mut Self {
        self.select(PlotKind::Coefficient)
    }

    /// Terminate the builder: select a forest plot.
    pub fn forest(&mut self) -> &mut Self {
        self.select(PlotKind::Forest)
    }

    /// Terminate the builder: select a horizontal paired-endpoint dumbbell plot.
    pub fn dumbbell(&mut self) -> &mut Self {
        self.select(PlotKind::Dumbbell)
    }

    /// Terminate the builder: select a categorical heatmap.
    pub fn heatmap(&mut self) -> &mut Self {
        self.select(PlotKind::Heatmap)
    }

    /// Terminate the builder: select the bar plot.
    pub fn bar(&mut self) -> &mut Self {
        self.select(PlotKind::Bar)
    }

    /// Terminate the builder: select the horizontal bar plot.
    pub fn horizontal_bar(&mut self) -> &mut Self {
        self.select(PlotKind::HorizontalBar)
    }

    /// Terminate the builder: select the stacked bar plot.
    pub fn stacked_bar(&mut self) -> &mut Self {
        self.select(PlotKind::StackedBar)
    }

    /// Terminate the builder: select the stacked horizontal bar plot.
    pub fn stacked_horizontal_bar(&mut self) -> &mut Self {
        self.select(PlotKind::StackedHorizontalBar)
    }

    /// Terminate the builder: select the boxplot.
    pub fn boxplot(&mut self) -> &mut Self {
        self.select(PlotKind::Boxplot)
    }

    /// Terminate the builder: select the violin plot.
    pub fn violin(&mut self) -> &mut Self {
        self.select(PlotKind::Violin)
    }

    /// Terminate the builder: select the regression plot.
    pub fn regression(&mut self) -> &mut Self {
        self.select(PlotKind::Regression)
    }

    /// Terminate the builder: select the marginal plot.
    pub fn marginal(&mut self) -> &mut Self {
        self.select(PlotKind::Marginal)
    }

    /// Terminate the builder: select the matrix plot.
    pub fn matrix(&mut self) -> &mut Self {
        self.select(PlotKind::Matrix)
    }

    /// Terminate the builder: select the mass-spec spectrum
    /// plot. Renders the `(x_col, y_col)` trace as a line
    /// series with peak markers from `add_peaks()` overlaid.
    /// Returns `&mut Self` so the caller can immediately call
    /// `render_to_svg` / `render_to_png`.
    pub fn mass_spectrum(&mut self) -> &mut Self {
        self.select(PlotKind::MassSpectrum)
    }

    /// Terminate the builder: select a beeswarm plot.
    pub fn beeswarm(&mut self) -> &mut Self {
        self.select(PlotKind::Beeswarm)
    }

    /// Terminate the builder: select a waterfall plot.
    pub fn waterfall(&mut self) -> &mut Self {
        self.select(PlotKind::Waterfall)
    }
}
