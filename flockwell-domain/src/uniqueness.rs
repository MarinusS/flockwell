//! The tag namespaces subject to uniqueness, shared by audits and mutations.
use crate::{AnimalData, Tag, UhfTag};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum UniqueTag {
    Tag(Tag),
    UhfTag(UhfTag),
}

pub(crate) fn unique_tags(data: &AnimalData) -> impl Iterator<Item = UniqueTag> + use<> {
    [
        data.tag.clone().map(UniqueTag::Tag),
        data.uhf_tag.clone().map(UniqueTag::UhfTag),
    ]
    .into_iter()
    .flatten()
}
