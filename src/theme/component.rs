//! Shared construction rules for complete, privately stored role values.

macro_rules! field_type {
    (optional $ty:ty) => { Option<$ty> };
    ($kind:ident $ty:ty) => { $ty };
}

macro_rules! field_getter {
    (borrow $field:ident: $ty:ty) => {
        #[allow(
            dead_code,
            reason = "complete private inspection awaits Theme customization"
        )]
        pub(crate) fn $field(&self) -> &$ty {
            &self.$field
        }
    };
    (copy $field:ident: $ty:ty) => {
        #[allow(
            dead_code,
            reason = "complete private inspection awaits Theme customization"
        )]
        pub(crate) fn $field(&self) -> $ty {
            self.$field
        }
    };
    (optional $field:ident: $ty:ty) => {
        #[allow(
            dead_code,
            reason = "complete private inspection awaits Theme customization"
        )]
        pub(crate) fn $field(&self) -> Option<&$ty> {
            self.$field.as_ref()
        }
    };
}

macro_rules! validated_component {
    ($name:ident { $($kind:ident $field:ident: $ty:ty => $edit:ident),+ $(,)? }
        validate |$value:ident| $validation:block) => {
        #[derive(Clone, Debug, PartialEq)]
        pub(crate) struct $name {
            $($field: super::component::field_type!($kind $ty)),+
        }

        impl $name {
            #[allow(clippy::too_many_arguments, reason = "the complete constructor follows the approved typed role schema")]
            pub(crate) fn new($($field: super::component::field_type!($kind $ty)),+) -> $crate::Result<Self> {
                let value = Self { $($field),+ };
                value.validate()?;
                Ok(value)
            }

            #[allow(clippy::unnecessary_wraps, reason = "immutable scalar groups share complete recursive validation")]
            pub(crate) fn validate(&self) -> $crate::Result<()> {
                let $value = self;
                $validation
                Ok(())
            }

            $(
                super::component::field_getter!($kind $field: $ty);

                #[allow(dead_code, reason = "complete private construction edits await Theme customization")]
                pub(crate) fn $edit(mut self, value: super::component::field_type!($kind $ty)) -> $crate::Result<Self> {
                    self.$field = value;
                    self.validate()?;
                    Ok(self)
                }
            )+
        }
    };
}

pub(super) use {field_getter, field_type, validated_component};
