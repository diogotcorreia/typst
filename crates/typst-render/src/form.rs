use tiny_skia as sk;
use typst_library::layout::Frame;
use typst_library::model::{FieldAppearance, FieldAppearanceKind};

use crate::{State, render_frame};

/// Render a (non-interactive) form field into the canvas.
pub fn render_field(
    canvas: &mut sk::Pixmap,
    state: State,
    appearance: &FieldAppearance,
    frame: &Frame,
) {
    // TODO: select "On" appearance when checkbox/radio is checked
    match appearance.kind {
        FieldAppearanceKind::Single | FieldAppearanceKind::Off => {
            render_frame(canvas, state, frame);
        }
        FieldAppearanceKind::On => {}
    }
}
