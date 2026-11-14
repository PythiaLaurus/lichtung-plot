//! Raw categorical identity is independent of display aliases and palette order.

use std::sync::Arc;

use crate::{Error, Result};

pub(crate) fn shared_text(source: &str) -> Result<Arc<str>> {
    let bytes = std::alloc::Layout::array::<u8>(source.len())
        .map_err(|_| Error::Render("guide identity text layout overflow".into()))?;
    std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(bytes)
        .map_err(|_| Error::Render("guide identity Arc layout overflow".into()))?;
    Ok(Arc::from(super::source::copy_source_string(source)?))
}

macro_rules! raw_identity {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub(crate) struct $name(Arc<str>);

        impl $name {
            pub(crate) fn new(value: &str) -> Result<Self> {
                shared_text(value).map(Self)
            }

            pub(crate) fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

raw_identity!(RawField);
raw_identity!(RawCategory);

impl RawField {
    pub(crate) fn from_shared(value: Arc<str>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CategoryKey {
    field: RawField,
    value: RawCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CategoryRef<'a> {
    pub(crate) field: &'a str,
    pub(crate) value: &'a str,
}

impl CategoryKey {
    /// Share an already-read source value without introducing guide validation.
    pub(crate) fn new(field: RawField, value: impl Into<Arc<str>>) -> Self {
        Self {
            field,
            value: RawCategory(value.into()),
        }
    }

    pub(crate) fn as_ref(&self) -> CategoryRef<'_> {
        CategoryRef {
            field: self.field.as_str(),
            value: self.value.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_identity_compares_raw_field_and_value_across_independent_storage() {
        let field = || RawField::from_shared(Arc::from("cohort"));
        let first = CategoryKey::new(field(), String::from(" first "));
        let separately_owned = CategoryKey::new(field(), String::from(" first "));
        assert!(!Arc::ptr_eq(&first.value.0, &separately_owned.value.0));
        assert_eq!(first, separately_owned);
        assert_ne!(first, CategoryKey::new(field(), "first"));
        assert_ne!(
            first,
            CategoryKey::new(RawField::from_shared(Arc::from("other")), " first ")
        );
        assert_eq!(first.as_ref().field, "cohort");
        assert_eq!(first.as_ref().value, " first ");
    }
}
