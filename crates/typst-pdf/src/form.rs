use std::cell::RefCell;
use std::sync::Arc;

use krilla::action::{Action, LinkAction};
use krilla::annotation::{Target, WidgetAnnotationKind};
use krilla::destination::XyzDestination;
use krilla::form as kf;
use krilla::form::FieldKind;
use krilla::geom as kg;
use krilla::page::Page;
use krilla::stream::Stream;
use krilla::surface::Surface;
use krilla::tagging::Identifier;
use typst_library::diag::{At, ExpectInternal, SourceResult, bail};
use typst_library::introspection::PagedPosition;
use typst_library::layout::{Abs, Frame, Point, Sides, Size};
use typst_library::model::{
    Destination, FieldAppearance, FieldAppearanceKind, FormField, ResolvedLink,
};
use typst_syntax::Span;

use crate::convert::{FrameContext, GlobalContext, PageIndexConverter, handle_frame};
use crate::tags::{self, AnnotationId, GroupId};
use crate::util::PointExt;

pub(crate) struct Field {
    pub krilla_field: FieldKind,
}

impl Field {
    pub(crate) fn new(field: FormField) -> Self {
        let field = match field {
            FormField::Checkbox(checkbox_field) => {
                kf::FormField::checkbox(checkbox_field.name.to_string(), false)
            }
        };

        Self { krilla_field: field.into() }
    }

    pub(crate) fn insert_annotation(
        &mut self,
        page: &mut Page,
        annotation: WidgetAnnotation,
    ) -> Identifier {
        match &mut self.krilla_field {
            FieldKind::PushButton(..) => todo!(),
            FieldKind::Checkbox(form_field) => {
                let widget = form_field.new_widget(
                    annotation.bbox,
                    annotation.off_stream.expect("checkbox has no off appearance"),
                    annotation.on_stream.expect("checkbox has no on appearance"),
                );

                page.add_widget_annotation(form_field, widget.into())
            }
            FieldKind::Radio(..) => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct WidgetAnnotation {
    pub bbox: kg::Rect,
    pub on_stream: Option<Stream>,
    pub off_stream: Option<Stream>,
}

impl WidgetAnnotation {
    pub fn new(bbox: kg::Rect) -> Self {
        Self { bbox, on_stream: None, off_stream: None }
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

    let widget = fc.get_widget_annotation_mut(appearance.name.clone(), rect);
    match appearance.kind {
        FieldAppearanceKind::Single | FieldAppearanceKind::Off => {
            debug_assert!(widget.off_stream.is_none());
            widget.off_stream = Some(stream);
        }
        FieldAppearanceKind::On => {
            debug_assert!(widget.on_stream.is_none());
            widget.on_stream = Some(stream);
        }
    }

    Ok(())
}

pub(crate) fn handle_form_field(
    fc: &mut FrameContext,
    gc: &mut GlobalContext,
    surface: &mut Surface,
    form_field: &FormField,
    size: Size,
) -> SourceResult<()> {
    match form_field {
        FormField::Checkbox(checkbox_field) => {
            if !gc.fields.contains_key(&checkbox_field.name) {
                let field = Field::new(form_field.clone());
                gc.fields.insert(checkbox_field.name.clone(), field);
            }
        }
    };

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

/// Turns a position link into a PDF XYZ destination.
///
/// - Takes into account page index conversion (if only part of the document is
///   exported)
/// - Consistently shifts the link by 10pt because the position of e.g.
///   backlinks to footnotes is always at the baseline and if you link directly
///   to it, the text will not be visible since it is right above.
pub(crate) fn pos_to_xyz(
    pic: &PageIndexConverter,
    pos: PagedPosition,
) -> Option<XyzDestination> {
    let page_index = pic.pdf_page_index(pos.page.get() - 1)?;
    let adjusted =
        Point::new(pos.point.x, (pos.point.y - Abs::pt(10.0)).max(Abs::zero()));
    Some(XyzDestination::new(page_index, adjusted.to_krilla()))
}
