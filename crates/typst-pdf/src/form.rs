use ecow::EcoString;
use krilla::form as kf;
use krilla::geom as kg;
use krilla::page::Page;
use krilla::stream::Stream;
use krilla::surface::Surface;
use krilla::tagging::Identifier;
use rustc_hash::FxHashMap;
use std::iter::Peekable;
use typst_library::diag::{At, ExpectInternal, SourceResult, bail};
use typst_library::layout::{Abs, Frame, Point, Sides, Size};
use typst_library::model::{FieldAppearance, FieldAppearanceKind, FormField};
use typst_syntax::Span;

use crate::convert::{FrameContext, GlobalContext, handle_frame};
use crate::tags::{self, AnnotationId};
use crate::util::PointExt;

pub(crate) struct Field {
    pub name: EcoString,
    pub krilla_field: kf::FieldKind,
}

impl Field {
    pub(crate) fn new(field: FormField) -> Self {
        let full_name = field.name().clone();
        let partial_name = full_name.rsplit('.').next().unwrap();
        let field = match field {
            FormField::Checkbox(checkbox_field) => {
                kf::FormField::checkbox(partial_name.to_string(), checkbox_field.checked)
                    .with_read_only(checkbox_field.read_only)
            }
        };

        Self { name: full_name, krilla_field: field.into() }
    }

    pub(crate) fn insert_annotation(
        &mut self,
        page: &mut Page,
        annotation: WidgetAnnotation,
    ) -> Identifier {
        match &mut self.krilla_field {
            kf::FieldKind::PushButton(..) => todo!(),
            kf::FieldKind::Checkbox(form_field) => {
                let widget = form_field.new_widget(
                    annotation.bbox,
                    annotation.off_stream.expect("checkbox has no off appearance"),
                    annotation.on_stream.expect("checkbox has no on appearance"),
                );

                page.add_widget_annotation(form_field, widget.into())
            }
            kf::FieldKind::Radio(..) => todo!(),
            kf::FieldKind::Text(..) => todo!(),
            kf::FieldKind::ListBox(..) => todo!(),
            kf::FieldKind::ComboBox(..) => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct WidgetAnnotation {
    pub annotation_id: Option<AnnotationId>,
    pub bbox: kg::Rect,
    pub on_stream: Option<Stream>,
    pub off_stream: Option<Stream>,
}

impl WidgetAnnotation {
    pub fn new(bbox: kg::Rect) -> Self {
        Self {
            annotation_id: None,
            bbox,
            on_stream: None,
            off_stream: None,
        }
    }
}

pub(crate) fn handle_field_appearance(
    fc: &mut FrameContext,
    gc: &mut GlobalContext,
    surface: &mut Surface,
    appearance: &FieldAppearance,
    body: &Frame,
) -> SourceResult<()> {
    let rect = bounding_box(fc, body.size());

    let stream = {
        let mut builder = surface.stream_builder();
        let mut fc = FrameContext::new(None, body.size());
        handle_frame(
            &mut fc,
            body,
            Sides::splat(Abs::zero()),
            None,
            &mut builder.surface(),
            gc,
        )?;

        builder.finish()
    };

    let tag_group = if tags::disabled(gc) {
        if gc.tags.in_tiling
            && let Some(accessibility) = gc.options.validators().accessibility()
        {
            let validator = accessibility.as_str();
            bail!(
                Span::detached(),
                "{validator} error: PDF artifacts may not contain form fields";
                hint: "a form field was used within a tiling";
            );
        }

        None
    } else {
        let (group_id, form_field) = gc
            .tags
            .tree
            .parent_form_field()
            .expect_internal("expected form field ancestor in logical tree")
            .at(Span::detached())?;

        if gc.tags.tree.parent_artifact().is_some() {
            if let Some(accessibility) = gc.options.validators().accessibility() {
                let validator = accessibility.as_str();
                bail!(
                    form_field.span(),
                    "{validator} error: PDF artifacts may not contain form fields";
                );
            }

            None
        } else {
            Some((group_id, form_field))
        }
    };

    let widget = fc.get_widget_annotation_mut(appearance.name.clone(), rect);
    match appearance.kind {
        FieldAppearanceKind::Single | FieldAppearanceKind::Off => {
            debug_assert!(widget.off_stream.is_none());
            widget.off_stream = Some(stream);

            if let Some((group_id, _)) = tag_group {
                let annot_id = gc.tags.annotations.reserve();
                widget.annotation_id = Some(annot_id);
                let group = gc.tags.tree.groups.get_mut(group_id);
                group.push_annotation(annot_id);
            }
        }
        FieldAppearanceKind::On => {
            debug_assert!(widget.on_stream.is_none());
            widget.on_stream = Some(stream);
        }
    }

    Ok(())
}

pub(crate) fn handle_form_field(
    gc: &mut GlobalContext,
    form_field: &FormField,
) -> SourceResult<()> {
    match form_field {
        FormField::Checkbox(checkbox_field) => {
            if !gc.fields.contains_key(&checkbox_field.name) {
                let field = Field::new(form_field.clone());
                gc.fields.insert(checkbox_field.name.clone(), field);
            }
        }
    }

    Ok(())
}

/// Compute the bounding box of the transformed rectangle for this frame.
fn bounding_box(fc: &FrameContext, size: Size) -> kg::Rect {
    let pos = Point::zero();
    let points = [
        pos + Point::with_y(size.y),
        pos + size.to_point(),
        pos + Point::with_x(size.x),
        pos,
    ];

    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for point in points {
        let p = point.transform(fc.state().transform()).to_krilla();
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }

    kg::Rect::from_ltrb(min_x, min_y, max_x, max_y).unwrap()
}

type FieldGroup = FxHashMap<EcoString, Node>;

enum Node {
    Group(FieldGroup),
    Leaf(Field),
}

pub(crate) fn build_field_tree(gc: &mut GlobalContext) -> SourceResult<kf::FieldTree> {
    fn insert_into_tree<'a, I>(
        fields: &mut FieldGroup,
        mut path: Peekable<I>,
        field: Field,
    ) -> SourceResult<()>
    where
        I: Iterator<Item = &'a str>,
    {
        let path_segment =
            path.next().expect("there is always one path segment when splitting");
        let is_leaf = path.peek().is_none();

        if is_leaf {
            let name = field.name.clone();
            if fields.insert(path_segment.into(), Node::Leaf(field)).is_some() {
                // TODO span?
                bail!(
                    Span::detached(), "there are two distinct form fields named `{name}`";
                    hint: "this can happen if you have a field named `{name}` and another named `{name}.<something else>`"
                )
            }
        } else {
            let node = fields
                .entry(path_segment.into())
                .or_insert_with(|| Node::Group(FieldGroup::default()));
            match node {
                Node::Group(group) => {
                    insert_into_tree(group, path, field)?;
                }
                Node::Leaf(field) => {
                    let name = &field.name;
                    // TODO span?
                    bail!(
                        Span::detached(), "there are two distinct form fields named `{name}`";
                        hint: "this can happen if you have a field named `{name}` and another named `{name}.<something else>`"
                    )
                }
            }
        }

        Ok(())
    }

    fn to_krilla_tree(group: FieldGroup) -> Vec<kf::Node> {
        group
            .into_iter()
            .map(|(name, node)| match node {
                Node::Group(group) => {
                    let fields = to_krilla_tree(group);
                    kf::Node::Group(kf::FieldGroup { name: name.to_string(), fields })
                }
                Node::Leaf(field) => kf::Node::Leaf(field.krilla_field),
            })
            .collect()
    }

    let mut fields = FieldGroup::default();
    for field in std::mem::take(&mut gc.fields).into_values() {
        let name = field.name.clone();
        let path = name.split('.').peekable();
        insert_into_tree(&mut fields, path, field)?;
    }

    Ok(kf::FieldTree { fields: to_krilla_tree(fields) })
}
