use krilla::tagging::{self as kt, Tag, TagKind};
use smallvec::SmallVec;
use typst_library::foundations::{Content, Packed};
use typst_library::model::{FigureElem, FormWidget};

use crate::tags::context::{FigureId, FormFieldId};
use crate::tags::groups::{Group, GroupId, GroupKind, Groups};
use crate::tags::resolve::TagNode;
use crate::tags::tree::Tree;
use crate::tags::util::PropertyOptRef;

#[derive(Debug, Clone, PartialEq)]
pub struct FormFieldCtx {
    pub group_id: GroupId,
    pub elem: Content,
    pub labels: SmallVec<[GroupId; 1]>,
}

impl FormFieldCtx {
    pub fn new(group_id: GroupId, elem: Content) -> Self {
        Self { group_id, elem, labels: SmallVec::new() }
    }

    pub fn build_tag(&self) -> Option<TagKind> {
        Some(Tag::Div.with_placement(Some(kt::Placement::Block)).into())
    }

    /// Generate an enclosing `Div` tag if there is a caption.
    pub fn build_wrapper_tag(&self) -> Option<TagKind> {
        (!self.labels.is_empty()).then_some(Tag::Div.into())
    }
}

pub fn build_form_field(tree: &mut Tree, form_field_id: FormFieldId) {
    let form_field_ctx = tree.ctx.form_fields.get_mut(form_field_id);
    let group = tree.groups.get(form_field_ctx.group_id);
    let wrapper = group.parent;

    // Insert the captions inside the figure.
    // an enclosing element.
    if !form_field_ctx.labels.is_empty() {
        for &caption in &form_field_ctx.labels {
            tree.groups.get_mut(caption).parent = wrapper;
        }
        tree.groups.prepend_groups(wrapper, &form_field_ctx.labels);
    }
}

fn single_semantic_child<'a>(
    groups: &'a Groups,
    mut group: &'a Group,
) -> Option<GroupId> {
    while let [TagNode::Group(id)] = group.nodes() {
        group = groups.get(*id);
        if group.kind.is_semantic() {
            return Some(*id);
        }
    }
    None
}
