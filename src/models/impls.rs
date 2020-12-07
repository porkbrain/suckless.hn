use {
    serde::Serializer,
    std::{borrow::Cow, cmp::Ordering, fmt, result},
};

use {super::*, crate::filter::Filter};

impl Modifier {
    fn inner_filter(&self) -> FilterKind {
        match self {
            Self::With(filter) => *filter,
            Self::Without(filter) => *filter,
        }
    }
}

/// If a filter is prepended with "+", it means that we want to include stories
/// which were flagged by the filter. "-" means exclude. This custom
/// serialization implementation follows the convention.
impl Serialize for Modifier {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for FilterKind {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

impl fmt::Display for FilterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for FilterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::With(filter) => write!(f, "+{}", filter.name()),
            Self::Without(filter) => write!(f, "-{}", filter.name()),
        }
    }
}

impl fmt::Debug for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Ord for FilterKind {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(other.name())
    }
}

impl PartialOrd for FilterKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Modifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner_filter().cmp(&other.inner_filter())
    }
}

impl PartialOrd for Modifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Theme {
    /// Given object name, returns the path where should the object be stored
    /// in the S3.
    pub fn object_path<'a>(&self, object_name: &'a str) -> Cow<'a, str> {
        match self {
            // default view is dark hence path prefix
            Self::Dark => Cow::Borrowed(object_name),
            Self::Light => Cow::Owned(format!("light/{}", object_name)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
