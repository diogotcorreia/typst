use std::cell::RefCell;
use std::sync::Arc;

use krilla::action::{Action, LinkAction};
use krilla::annotation::{Target, WidgetAnnotationKind};
use krilla::destination::XyzDestination;
use krilla::form::FieldKind;
use krilla::geom as kg;
use krilla::surface::Surface;
use typst_library::diag::{At, ExpectInternal, SourceResult, bail};
use typst_library::introspection::PagedPosition;
use typst_library::layout::{Abs, Point, Size};
use typst_library::model::{Destination, FormField, ResolvedLink};
use typst_syntax::Span;

use crate::convert::{FrameContext, GlobalContext, PageIndexConverter};
use crate::tags::{self, AnnotationId, GroupId};
use crate::util::PointExt;

// pub(crate) struct WidgetAnnotation {
//     pub kind: LinkAnnotationKind,
//     pub alt: Option<String>,
//     pub span: Span,
//     pub rects: Vec<kg::Rect>,
//     pub target: Target,
// }
//
// pub(crate) enum WidgetAnnotationKind {
//     /// A link annotation that is tagged within a `Link` structure element.
//     Tagged(AnnotationId),
//     /// A link annotation within an artifact.
//     Artifact,
// }

pub(crate) fn handle_form_field(
    fc: &mut FrameContext,
    gc: &mut GlobalContext,
    surface: &mut Surface,
    form_field: &FormField,
    size: Size,
) -> SourceResult<()> {
    let rect = bounding_box(fc, size);
    let (field, annotation) = match form_field {
        FormField::Checkbox(checkbox_field) => {
            let field =
                krilla::form::FormField::checkbox(checkbox_field.name.to_string(), false);

            let ap = {
                let mut builder = surface.stream_builder();
                builder.surface().finish();
                builder.finish()
            };

            let widget = field.new_widget(rect, ap.clone(), ap.clone());

            (FieldKind::from(field), WidgetAnnotationKind::from(widget))
        }
    };

    let field = Arc::new(RefCell::new(field));
    fc.push_widget_annotation(field.clone(), annotation);

    gc.fields.push(field);


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
