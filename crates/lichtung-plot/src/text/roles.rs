//! Deliberate owner-level defaults; authored requests replace these whole values.

use crate::TextLayout;

/// An actual guide text role, independent of its shared Theme appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideTextRole {
    /// Section titles.
    Title,
    /// Discrete entries and named guide keys.
    Entry,
    /// Continuous numeric labels and their explanation.
    Number,
}
impl GuideTextRole {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Title => 0,
            Self::Entry => 1,
            Self::Number => 2,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TextRoles(std::sync::Arc<RoleInstructions>);

impl std::ops::Deref for TextRoles {
    type Target = RoleInstructions;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TextRoles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        std::sync::Arc::make_mut(&mut self.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RoleInstructions {
    pub(crate) ticks: [Option<TextLayout>; 2],
    pub(crate) facet: [Option<TextLayout>; 2],
    pub(crate) matrix: [Option<TextLayout>; 2],
    pub(crate) guide: [Option<TextLayout>; 3],
    pub(crate) value: Option<TextLayout>,
    pub(crate) null_row: Option<TextLayout>,
    pub(crate) estimate: Option<TextLayout>,
    pub(crate) total: Option<TextLayout>,
    pub(crate) significance: Option<TextLayout>,
}
