//! Pairwise visual annotations supplied by callers.

/// Visual marker for a significance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignificanceMark {
    /// Not significant. Rendered as `ns`.
    Ns,
    /// Rendered as `*`.
    One,
    /// Rendered as `**`.
    Two,
    /// Rendered as `***`.
    Three,
    /// Rendered as `****`.
    Four,
}

impl SignificanceMark {
    /// Map a p-value to the conventional significance tier.
    #[must_use]
    pub fn from_p_value(p_value: f64) -> Self {
        if !p_value.is_finite() || p_value >= 0.05 {
            Self::Ns
        } else if p_value < 0.0001 {
            Self::Four
        } else if p_value < 0.001 {
            Self::Three
        } else if p_value < 0.01 {
            Self::Two
        } else {
            Self::One
        }
    }

    /// Return the conventional display glyph.
    #[must_use]
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Ns => "ns",
            Self::One => "*",
            Self::Two => "**",
            Self::Three => "***",
            Self::Four => "****",
        }
    }
}

/// One endpoint of a pairwise visual annotation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComparisonEndpoint {
    /// Category level on the categorical axis.
    pub category: String,
    /// Optional graphical-group level identifying a dodge anchor.
    pub group: Option<String>,
    /// Optional hue level validating the selected mark's color identity.
    pub hue: Option<String>,
}

impl ComparisonEndpoint {
    /// Construct a category-only endpoint.
    #[must_use]
    pub fn new(category: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            group: None,
            hue: None,
        }
    }

    /// Qualify this endpoint with a graphical group.
    #[must_use]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Qualify this endpoint with a hue level.
    #[must_use]
    pub fn with_hue(mut self, hue: impl Into<String>) -> Self {
        self.hue = Some(hue.into());
        self
    }
}

/// One variable-level selector identifying an annotation's facet panel.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComparisonFacet {
    /// Facet column name.
    pub variable: String,
    /// Facet level rendered in the owning panel.
    pub level: String,
}

/// Owned data for one explicit pairwise visual annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonAnnotation {
    /// First endpoint.
    pub endpoint_a: ComparisonEndpoint,
    /// Second endpoint.
    pub endpoint_b: ComparisonEndpoint,
    /// Facet cells that own this comparison; empty for an unfaceted chart.
    pub facet: Vec<ComparisonFacet>,
    /// Method-native p-value.
    pub raw_p_value: f64,
    /// Adjusted or intrinsically familywise p-value, when applicable.
    pub adjusted_p_value: Option<f64>,
    /// Human-readable correction label, when a separate correction was applied.
    pub correction: Option<String>,
}

impl ComparisonAnnotation {
    /// Construct a category-only comparison with its method-native p-value.
    #[must_use]
    pub fn new(group_a: impl Into<String>, group_b: impl Into<String>, p_value: f64) -> Self {
        Self {
            endpoint_a: ComparisonEndpoint::new(group_a),
            endpoint_b: ComparisonEndpoint::new(group_b),
            facet: Vec::new(),
            raw_p_value: p_value,
            adjusted_p_value: None,
            correction: None,
        }
    }

    /// Attach an adjusted p-value and correction label.
    #[must_use]
    pub fn adjusted(mut self, p_value: f64, correction: impl Into<String>) -> Self {
        self.adjusted_p_value = Some(p_value);
        self.correction = Some(correction.into());
        self
    }

    /// Mark a p-value as intrinsically familywise.
    #[must_use]
    pub fn familywise(mut self, p_value: f64) -> Self {
        self.adjusted_p_value = Some(p_value);
        self.correction = None;
        self
    }

    /// Restrict this comparison to one named facet cell.
    #[must_use]
    pub fn in_facet(mut self, variable: impl Into<String>, level: impl Into<String>) -> Self {
        let facet = ComparisonFacet {
            variable: variable.into(),
            level: level.into(),
        };
        if !self.facet.contains(&facet) {
            self.facet.push(facet);
        }
        self
    }

    /// Return the p-value used for the visual mark.
    #[must_use]
    pub fn display_p_value(&self) -> f64 {
        self.adjusted_p_value.unwrap_or(self.raw_p_value)
    }

    /// Return the significance tier used for the visual mark.
    #[must_use]
    pub fn render_mark(&self) -> SignificanceMark {
        SignificanceMark::from_p_value(self.display_p_value())
    }

    /// Return whether the displayed p-value is below 0.05.
    #[must_use]
    pub fn is_significant(&self) -> bool {
        self.display_p_value().is_finite() && self.display_p_value() < 0.05
    }
}
