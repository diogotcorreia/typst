use std::sync::Arc;

use comemo::Track;
use ecow::{EcoString, EcoVec, eco_format, eco_vec};
use rustc_hash::FxHashMap;
use typst_macros::{Cast, cast};
use typst_syntax::Span;
use typst_utils::singleton;

use crate::diag::{
    At, CollectCombinedResult, Hint, HintedStrResult, SourceResult, StrResult, bail,
    error,
};
use crate::engine::Engine;
use crate::foundations::{
    Args, Array, Construct, Content, Context, Dict, FromValue, IntoValue, Label,
    NativeElement, OneOrMultiple, Packed, Repr, Selector, ShowSet, Smart, StyleChain,
    Styles, Synthesize, Value, dict, elem, scope,
};
use crate::introspection::{
    Counter, CounterKey, Introspector, Location, QueryIntrospection,
    QueryLabelIntrospection,
};
use crate::layout::{Abs, Frame, Length, Rel, Sides, Size, Sizing};
use crate::visualize::{CircleElem, Paint, RectElem};

/// A form with interactive and fillable fields.
///
/// Forms fields provide a way for users to users to interact with your document.
/// For example, a user can fill out their name on a textbox
/// or indicate their dietary restrictions using checkboxes. // TODO better examples?
/// Typst supports many field types across both PDF and HTML,
/// such as @form.button[buttons], @form.checkbox[checkboxes],
/// @form.radio-group[radio buttons], @form.textbox[textboxes], @form.choice[dropdowns],
/// and @form.choice[listboxes].
/// When exporting to PDF, signature fields are also supported.
/// For a hands-on explanation of all the ways you can use and customize form fields
/// in Typst, check out the @guides:forms[Form Guide].
///
/// All form fields in a Typst document must be enclosed within a @form element.
///
/// = Example <example>
/// The example below demonstrates creating a simple form with various form fields.
/// However, this example and the other ones on this page are not interactive.
///
/// ```example
/// #form[
///   #form.label(<name>)[Name:]
///   #form.textbox() <name>
///
///   Food preferences:
///   #grid(
///     columns: 2,
///     gutter: 0.4em,
///     align: horizon,
///     [#form.checkbox() <meat>],
///     form.label(<meat>)[Meat],
///     [#form.checkbox() <vegetarian>],
///     form.label(<vegetarian>)[Vegetarian],
///     [#form.checkbox() <vegan>],
///     form.label(<vegan>)[Vegan],
///     [#form.checkbox() <lactose>],
///     form.label(<lactose>)[Lactose-free],
///     [#form.checkbox() <gluten>],
///     form.label(<gluten>)[Gluten-free],
///   )
/// ]
/// ```
///
/// = Accessibility <accessibility>
///
/// Form fields need extra information to be properly consumed by users
/// of Assistive Technology (AT).
/// You can describe a form field via the @form.label element, which is
/// visible to sighted users and announced for users of AT.
/// We strongly recommend that all fields have an associated @form.label.
///
/// Furthermore, the @form.radio-group[radio group]'s @form.radio-group.title[`title`]
/// property can also be used to describe the entire group, and is preferred
/// over the generic @form.label element.
///
/// = Compatibility <compatibility>
///
/// Not all PDF readers have full support for all interactive forms features.
/// The following compatibility table shows which features are available in
/// each reader we tested, and when exporting to HTML.
/// While testing, we used these PDF readers:
/// - Adobe Acrobat Reader (version ???)
/// - Pdfium (Chrome 156)
/// - PDF.js (Firefox 159)
/// - Foxit Reader (version ???)
/// - Apple Quartz/Preview/???
///
/// #let yes = emoji.checkmark
/// #let partial = emoji.warning
/// #let no = emoji.crossmark
///
/// #docs-table(
///   table.header[][Acrobat][Pdfium][PDF.js][Foxit][Apple][HTML],
///   [@form.target[Submission] in FDF],
///   yes, yes, no,
///   [#yes#footnote[
///     Foxit Reader incorrectly sends a `Content-Type: x-www-form-url-encoded` header
///     for FDF and PDF. When submitting in XFDF, it correctly sends
///     `Content-Type: application/vnd.adobe.xfdf`.] <foxit-submission>
///   ],
///   [?], no,
///
///   [@form.target[Submission] in XFDF or PDF],
///   yes, no, no,
///   [#yes#footnote(<foxit-submission>)],
///   [?], no,
///
///   [@form.target[Submission] in URL encoded],
///   yes, no, no, yes, [?], yes,
///
///   [@form.target[Submission] in multipart or plain text],
///   no, no, no, no, [?], yes,
///
///   [@form.widget.state[Hover/down appearances]],
///   yes,
///   [#partial#footnote[Pdfium supports hover/down states on checkboxes and
///     radio buttons, but not on regular buttons.]],
///   no, [?], [?], no,
///
///   [Text color in textboxes and dropdowns],
///   yes, yes, yes, [?], [?], no,
///
///   [@form.textbox.fixed-width[Fixed-width textboxes]],
///   yes, yes, yes, [?], [?], no,
///
///   [@form.textbox.placeholder[Placeholder in textboxes]],
///   no, no, no, no, no, yes,
/// )
///
/// Everything else is fully supported in all readers and in HTML.
///
#[elem(scope, since = "unreleased", Locatable, ShowSet)]
pub struct FormElem {
    /// The fields belonging to this form, along with any surrounding content.
    #[required]
    pub body: Content,

    /// Where to submit this form when a @form.button with a
    /// @form.button.action[submit action] is pressed.
    ///
    /// Can either be a URL as a string, or a dictionary with the following fields:
    /// - `url` (required): the URL where the form data is to be submitted.
    ///   Can also be a relative path when exporting to HTML.
    /// - `content-type` (optional, default `{auto}`): the format to submit form data in.
    ///   Can be one of `{auto}`, `{"url-encoded"}`, `{"multipart"}`, `{"plain"}`, `{"pdf"}`,
    ///   `{"fdf"}`, or `{"xfdf"}`.
    ///   When set to `{auto}`, `{"fdf"}` is used for PDF export and `{"url-encoded"}`
    ///   is used for HTML export.
    /// - `method` (optional, default `{"post"}`): the HTTP method to use when submitting
    ///   form data. Can be one of `{"post"}` or `{"get"}`.
    ///
    /// It is highly recommended to set each form field's name explicitly when
    /// the form is expected to be submitted, as fields won't be submitted at all
    /// in HTML and will have auto-generated names in PDF.
    ///
    /// ```typ
    /// #form(
    ///   target: (
    ///     url: "https://example.com/submission",
    ///     content-type: "multipart",
    ///     method: "post",
    ///   )
    /// )
    /// ```
    pub target: Option<FormTarget>,

    /// This form, to be inherited by field elements.
    #[internal]
    #[ghost]
    pub form: Option<Packed<FormElem>>,
}

impl ShowSet for Packed<FormElem> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the form itself available to its field. Should be
        // superseded by a proper ancestry mechanism in the future.
        let mut elem = self.clone();
        elem.materialize(styles);
        out.set(FormElem::form, Some(elem));
        out
    }
}

#[scope]
impl FormElem {
    #[elem]
    type FormButton;
    #[elem]
    type FormCheckbox;
    #[elem]
    type FormRadioGroup;
    #[elem]
    type FormRadioButton;
    #[elem]
    type FormTextbox;
    #[elem]
    type FormChoice;
    #[elem]
    type FormSignature;
    #[elem]
    type FormLabel;
    #[elem]
    type FormWidget;
    #[elem]
    type FormCursor;
}

impl FormElem {
    /// Get the ancestor form element that is present in the style chain, if any.
    /// Returns an error if not.
    pub fn try_from_styles<'a, T: NativeElement>(
        elem: &Packed<T>,
        styles: &'a StyleChain,
    ) -> SourceResult<&'a Packed<FormElem>> {
        styles.get_ref(FormElem::form).as_ref().ok_or_else(|| {
            eco_vec![error!(
                elem.span(),
                "this {} element must be placed inside a form element", T::ELEM.name();
                hint: "try surrounding this element with `#form[...]`";
            )]
        })
    }
}

impl Packed<FormElem> {
    /// Check if this form is inside another form;
    /// if so, returns an error.
    pub fn assert_not_inside_form(&self, styles: StyleChain) -> SourceResult<()> {
        if let Some(parent) = styles
            .get_all_ref(FormElem::form)
            .filter_map(|opt| opt.as_ref())
            .find(|el| el.location() != self.location())
        {
            bail!(
                self.span(), "this form cannot be inside another form";
                hint[parent.span()]: "the outer form is here";
            )
        }
        Ok(())
    }
}

/// Where and how to submit a filled out form.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FormTarget {
    /// The URL to send form data to.
    pub url: EcoString,
    /// The format to submit the form data in.
    pub content_type: Smart<ContentType>,
    /// The HTTP method to use when submitting the form.
    pub method: HttpMethod,
}

cast! {
    FormTarget,
    self => Value::Dict(self.into()),
    url: EcoString => Self {
        url,
        content_type: Smart::Auto,
        method: HttpMethod::default(),
    },
    mut dict: Dict => {
        // Get a value by key, accepting either non-existence or something
        // convertible to type T.
        fn take<T: FromValue>(dict: &mut Dict, key: &str) -> HintedStrResult<Option<T>> {
            dict.take(key).ok().map(|v| v.cast()).transpose()
        }

        let url = take(&mut dict, "url")?
            .ok_or("missing the required `url` entry in the form target")?;
        let content_type = take(&mut dict, "content-type")?.unwrap_or_default();
        let method = take(&mut dict, "method")?.unwrap_or_default();
        dict.finish(&["url", "content-type", "method"])?;
        Self { url,  content_type, method }
    },
}

impl From<FormTarget> for Dict {
    fn from(target: FormTarget) -> Self {
        dict! {
            "url" => target.url,
            "content-type" => target.content_type,
            "method" => target.method,
        }
    }
}

/// The submission format.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Cast)]
pub enum ContentType {
    /// URL encoded key-value pairs (`application/x-www-form-url-encoded`).
    UrlEncoded,
    /// Multipart form data (`multipart/form-data`).
    Multipart,
    /// Plain text (`text/plain`).
    Plain,
    /// The PDF file itself (`application/pdf`).
    Pdf,
    /// Forms Data Format (`application/fdf`).
    Fdf,
    /// XML Forms Data Format (`application/xfdf`).
    Xfdf,
}

/// The HTTP method.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Cast)]
pub enum HttpMethod {
    /// GET.
    Get,
    #[default]
    /// POST.
    Post,
}

/// An interactive button.
///
/// A button can perform an action when clicked, such as
/// resetting all fields in a form to their default values,
/// or submitting the form.
///
/// ```example
/// >>> #form[
/// #form.button(action: "reset")[Reset Fields]
/// >>> ]
/// ```
#[elem(name = "button", since = "unreleased", Locatable, Tagged, ShowSet)]
pub struct FormButton {
    /// The label of this button.
    #[required]
    pub body: Content,

    /// The name of this field.
    ///
    /// If `{auto}`, will be automatically generated as `{"field-<n>"}`
    /// when exporting to PDF.
    pub name: Smart<EcoString>,

    /// The action that is triggered when this button is clicked, if any.
    ///
    /// When set to `{"submit"}`, the surrounding form must have a @form.target[target].
    pub action: Option<ButtonAction>,

    /// Whether this button is disabled and cannot be interacted with.
    pub disabled: bool,

    /// The button's width.
    pub width: Smart<Rel<Length>>,

    /// The button's height.
    pub height: Sizing,
}

impl ShowSet for Packed<FormButton> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the field itself available to the `FieldWidget`. Should be
        // superseded by a proper ancestry mechanism in the future.
        let mut elem = self.clone().pack();
        elem.materialize(styles);
        out.set(FormWidget::parent, Some(elem));

        let disabled = self.disabled.get(styles);
        styles::set_rect_stroke(&mut out, disabled);
        if disabled {
            styles::set_disabled_styles(&mut out);
        }
        out.set(RectElem::inset, Sides::splat(Some(Abs::pt(5.0).into())));
        out
    }
}

/// Supported actions when clicking a button.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Cast)]
pub enum ButtonAction {
    /// Submit the filled out form to the form's @form.target[submission target].
    Submit,
    /// Reset all fields in this form to their default values.
    Reset,
}

/// An interactive checkbox.
///
/// A checkbox can be used for binary choices, for example "yes" and "no".
/// Checkboxes are toggled on and off when clicked.
///
/// ```example
/// >>> #form[
/// #form.checkbox(checked: false)
/// #form.checkbox(checked: true)
/// >>> ]
/// ```
#[elem(name = "checkbox", since = "unreleased", Locatable, Tagged, ShowSet)]
pub struct FormCheckbox {
    /// The name of this field.
    ///
    /// If `{auto}`, will be automatically generated as `{"field-<n>"}`
    /// when exporting to PDF.
    pub name: Smart<EcoString>,

    /// Whether this checkbox is checked by default.
    pub checked: bool,

    /// Whether this checkbox is required and must be checked.
    pub required: bool,

    /// Whether this checkbox is read-only and cannot be interacted with.
    pub disabled: bool,

    /// The checkbox's width.
    pub width: Smart<Rel<Length>>,

    /// The checkbox's height.
    pub height: Sizing,
}

impl ShowSet for Packed<FormCheckbox> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the field itself available to the `FieldWidget`. Should be
        // superseded by a proper ancestry mechanism in the future.
        // No need to materialize since the widget show-rule does not access any property.
        out.set(FormWidget::parent, Some(self.clone().pack()));

        let disabled = self.disabled.get(styles);
        styles::set_rect_stroke(&mut out, disabled);
        if disabled {
            styles::set_disabled_styles(&mut out);
        }
        out
    }
}

/// A group of radio buttons.
///
/// Radio groups can be used for exclusive choices.
/// In a group of radio buttons, at most one button can be selected at a time,
/// that is, when selecting one option, all other options are automatically unselected.
///
/// ```example
/// >>> #form[
/// #form.radio-group(
///   title: [Best typesetting software?],
///   selected: <typst>,
///   grid(
///     columns: 2,
///     gutter: 0.4em,
///     align: horizon,
///     [#form.radio() <typst>],
///     form.label(<typst>)[Typst],
///     [#form.radio() <latex>],
///     form.label(<latex>)[LaTeX],
///     [#form.radio() <md>],
///     form.label(<md>)[Markdown],
///   )
/// )
/// >>> ]
/// ```
#[elem(name = "radio-group", since = "unreleased", Locatable, ShowSet)]
pub struct FormRadioGroup {
    /// The radio buttons belonging to this group, along with any surrounding content.
    #[required]
    pub body: Content,

    /// The name of this field.
    ///
    /// If `{auto}`, will be automatically generated as `{"field-<n>"}`
    /// when exporting to PDF.
    pub name: Smart<EcoString>,

    /// The label pointing to the radio button that is selected by default.
    pub selected: Option<Label>,

    /// Whether this radio group is required and one of the options must be selected.
    pub required: bool,

    /// The title of this radio group.
    ///
    /// In PDF, this is also used as the alt text for this field.
    /// In HTML, this becomes the `<legend>` element of the `<fieldset>`.
    pub title: Option<Content>,

    /// Whether this radio group is read-only and none of its buttons
    /// can be interacted with.
    pub disabled: bool,
}

impl Packed<FormRadioGroup> {
    /// Resolves the selected radio button early.
    pub fn resolve_selected_early(
        &self,
        engine: &mut Engine,
        styles: StyleChain<'_>,
    ) -> SourceResult<Option<Packed<FormRadioButton>>> {
        let Some(selected) = self.selected.get_cloned(styles) else { return Ok(None) };
        let location = self.location().unwrap();
        let span = self.span();
        let matches = engine.introspect(QueryIntrospection(
            Selector::Label(selected).within(location.into()),
            span,
        ));
        Self::try_into_radio_button(selected, &matches, span).map(Some)
    }

    /// Resolves the selected radio button without an engine.
    pub fn resolve_selected_late(
        &self,
        introspector: &dyn Introspector,
    ) -> SourceResult<Option<Packed<FormRadioButton>>> {
        let Some(selected) = self.selected.get_cloned(StyleChain::default()) else {
            return Ok(None);
        };
        let location = self.location().unwrap();
        let span = self.span();
        let matches =
            introspector.query(&Selector::Label(selected).within(location.into()));
        Self::try_into_radio_button(selected, &matches, span).map(Some)
    }

    /// Attempts to convert the element returned by the query to a radio button,
    /// if there is only one.
    ///
    /// Returns an error instead if there is none or multiple,
    /// or if element is not a radio button.
    fn try_into_radio_button(
        selected: Label,
        matches: &[Content],
        span: Span,
    ) -> SourceResult<Packed<FormRadioButton>> {
        match matches {
            [content] => {
                if let Some(radio_button) = content.to_packed::<FormRadioButton>() {
                    Ok(radio_button.clone())
                } else {
                    bail!(
                        span, "label `{}` is not a radio button", selected.repr();
                        hint[content.span()]: "label refers to this {}", content.elem().name()
                    )
                }
            }
            [] => bail!(
                span,
                "label `{}` does not exist in the radio group",
                selected.repr()
            ),
            _ => bail!(
                span,
                "label `{}` occurs multiple times in the radio group",
                selected.repr()
            ),
        }
    }
}

impl ShowSet for Packed<FormRadioGroup> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the radio group itself available to its radio buttons. Should be
        // superseded by a proper ancestry mechanism in the future.
        let mut elem = self.clone();
        elem.materialize(styles);
        out.set(FormRadioButton::parent, Some(elem));
        out
    }
}

impl FormRadioGroup {
    /// Get the ancestor radio group element that is present in the style chain, if any.
    ///
    /// Returns an error if not.
    pub fn try_from_styles<'a, T: NativeElement>(
        elem: &Packed<T>,
        styles: &'a StyleChain,
    ) -> SourceResult<&'a Packed<FormRadioGroup>> {
        styles.get_ref(FormRadioButton::parent).as_ref().ok_or_else(|| {
            eco_vec![error!(
                elem.span(),
                "this radio button must be placed inside a radio group";
                hint: "try surrounding this button with `#form.radio-group[...]`";
            )]
        })
    }
}

/// An interactive radio button.
///
/// Must appear inside of a @form.radio-group[radio group].
#[elem(name = "radio", since = "unreleased", Locatable, Tagged, Synthesize, ShowSet)]
pub struct FormRadioButton {
    /// The value of the radio group when this option is selected.
    ///
    /// If `{auto}`, it is automatically generated on PDF export as `{"option-<n>"}`.
    pub value: Smart<EcoString>,

    /// The radio button's width.
    pub width: Smart<Rel<Length>>,

    /// The radio button's height.
    pub height: Sizing,

    /// Lets radio buttons access the radio group they are part of.
    /// This is a bit of a hack and should be superseded by a proper ancestry mechanism.
    #[ghost]
    #[internal]
    pub parent: Option<Packed<FormRadioGroup>>,

    /// Same as @form.radio.value[`value`], but with `{auto}` converted to an
    /// auto-generated value.
    #[internal]
    #[synthesized]
    pub resolved_value: EcoString,
}

impl Synthesize for Packed<FormRadioButton> {
    fn synthesize(
        &mut self,
        engine: &mut Engine,
        styles: StyleChain,
    ) -> SourceResult<()> {
        if let Smart::Custom(value) = self.value.get_ref(styles) {
            self.resolved_value = Some(value.clone());
            return Ok(());
        }

        if self.location().is_none() {
            return Ok(());
        }

        let span = self.span();

        let group = FormRadioGroup::try_from_styles(self, &styles)?;

        let selector = FormRadioButton::ELEM
            .select()
            .within(group.location().unwrap().into());
        let counter = Counter::new(CounterKey::Selector(selector));
        let context = Context::new(self.location(), Some(styles));
        let counter_state = counter.get(engine, context.track(), span)?;
        self.resolved_value = Some(eco_format!("option-{}", counter_state.first()));
        Ok(())
    }
}

impl ShowSet for Packed<FormRadioButton> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the field itself available to the `FieldWidget`. Should be
        // superseded by a proper ancestry mechanism in the future.
        // No need to materialize since the widget show-rule does not access any property.
        out.set(FormWidget::parent, Some(self.clone().pack()));

        let disabled = styles
            .get_ref(FormRadioButton::parent)
            .as_ref()
            .is_some_and(|group| group.disabled.get(styles));
        let stroke = styles::field_stroke(disabled);
        out.set(
            CircleElem::outset,
            Sides::splat(Some(Abs::pt(-styles::STROKE_THICKNESS_PT / 2.0).into())),
        );
        out.set(CircleElem::stroke, Smart::Custom(Some(stroke)));
        if disabled {
            out.set(CircleElem::fill, Some(styles::DISABLED_FILL.into()));
            styles::set_disabled_styles(&mut out);
        }
        out
    }
}

/// A free-form text input.
///
/// A textbox can be used when you want the user to input arbitrary text
/// into a field, such as their name, a long description, or some sort of identifier.
/// This field comes in three variants: single-line, multi-line, or fixed-width.
///
/// ```example
/// >>> #form[
/// >>> #set form.textbox(width: 100%)
/// // Single-line
/// #form.label(<name>)[Name:]
/// #form.textbox() <name>
///
/// // Multi-line
/// #form.label(<about>)[About:]
/// #form.textbox(multiline: true) <about>
///
/// // Fixed-width
/// #form.label(<id>)[ID number:]
/// #form.textbox(
///   fixed-width: true,
///   max-length: 9,
/// ) <id>
/// >>> ]
/// ```
#[elem(name = "textbox", since = "unreleased", Locatable, Tagged, ShowSet)]
pub struct FormTextbox {
    /// The name of this field.
    ///
    /// If `{auto}`, will be automatically generated as `{"field-<n>"}`
    /// when exporting to PDF.
    pub name: Smart<EcoString>,

    /// The default value of this field.
    pub value: Option<EcoString>,

    /// Whether the value of this field can span multiple lines.
    pub multiline: bool,

    /// Whether it is mandatory to fill out this field (i.e., it cannot be empty).
    pub required: bool,

    /// The max length, in characters, of the value of this field.
    pub max_length: Option<u32>,

    /// Whether this textbox is divided into as many equally spaced positions
    /// as the value of `max-length`.
    ///
    /// The `max-length` property must be set.
    /// Only supported in PDF export.
    pub fixed_width: bool,

    /// The placeholder text to show when the textbox is empty.
    ///
    /// Only supported in HTML export.
    pub placeholder: Option<EcoString>,

    /// Whether this text field is read-only and it cannot be interacted with.
    pub disabled: bool,

    /// Whether the spellchecker should be enabled when filling out this field.
    ///
    /// When set to `{auto}`, it is enabled in PDF export, and respects the browser
    /// defaults in HTML. In HTML export, this also controls text auto correction.
    pub spellcheck: Smart<bool>,

    /// The field's width.
    pub width: Smart<Rel<Length>>,

    /// The field's height.
    pub height: Sizing,
}

impl ShowSet for Packed<FormTextbox> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the field itself available to the `FieldWidget`. Should be
        // superseded by a proper ancestry mechanism in the future.
        let mut elem = self.clone().pack();
        elem.materialize(styles);
        out.set(FormWidget::parent, Some(elem));

        let disabled = self.disabled.get(styles);
        let stroke = styles::field_stroke(disabled);
        let stroke = if self.multiline.get(styles) {
            Sides::splat(Some(Some(stroke)))
        } else {
            Sides::new(None, None, None, Some(Some(stroke)))
        };

        out.set(
            RectElem::outset,
            Sides::splat(Some(Abs::pt(-styles::STROKE_THICKNESS_PT / 2.0).into())),
        );
        out.set(RectElem::stroke, Smart::Custom(stroke));
        if disabled {
            styles::set_disabled_styles(&mut out);
        }
        out
    }
}

/// A dropdown or listbox field.
///
/// Choice fields can be used when a user is expected to select from a pre-defined
/// list of options.
/// This field comes in two variants: a dropdown, when only one option can be selected
/// at a time, or a listbox when selecting multiple options is allowed.
///
/// ```example
/// >>> #form[
///  #let options = (
///    "Chocolate", "Vanilla",
///    "Raspberry", "Strawberry",
///  )
///
///  // Dropdown
///  #form.choice(
///     options: options,
///     value: "Chocolate",
///  )
///
///  // Listbox
///  #form.choice(
///     options: options,
///     multiple: true,
///     value: ("Chocolate", "Raspberry"),
///  )
/// >>> ]
/// ```
#[elem(name = "choice", since = "unreleased", Locatable, Tagged, Synthesize, ShowSet)]
pub struct FormChoice {
    /// The name of this field.
    ///
    /// If `{auto}`, will be automatically generated as `field-<n>`
    /// when exporting to PDF.
    pub name: Smart<EcoString>,

    /// Whether this field allows multiple options to be selected at the same time.
    ///
    /// - When `false`, this field is displayed as a dropdown.
    /// - When `true`, this field is displayed as a listbox.
    pub multiple: bool,

    /// The available options on this field.
    ///
    /// - If given a dictionary, the keys correspond to the value of each option,
    ///   while the values correspond to the user-facing display name
    ///   (or `{auto}` to use the value as the display name).
    /// - If given an array, its elements are treated as both the value of the
    ///   option and its display name.
    ///
    /// #example(
    ///   title: "Give an array of strings",
    ///   ```
    ///   >>> #form[
    ///   #form.choice(
    ///     options: ("red-car", "blue-car"),
    ///     value: "red-car",
    ///   )
    ///   >>> ]
    ///   ```
    /// )
    ///
    /// #example(
    ///   title: "Give a dictionary mapping to display name",
    ///   ```
    ///   >>> #form[
    ///   #form.choice(
    ///     options: (
    ///       "red-car": "Red Car",
    ///       "blue-car": "Blue Car",
    ///     ),
    ///     value: "red-car",
    ///   )
    ///   >>> ]
    ///   ```
    /// )
    pub options: ChoiceOptions,

    /// The default value of this field.
    ///
    /// Can be an array of string if @form.choice.multiple[`multiple`] is true.
    /// All entries must be present in @form.choice.options[`options`].
    pub value: OneOrMultiple<EcoString>,

    /// Whether it is mandatory to fill out this field (i.e., it cannot be empty).
    pub required: bool,

    /// Whether this text field is read-only and it cannot be interacted with.
    pub disabled: bool,

    /// The field's width.
    pub width: Smart<Rel<Length>>,

    /// The field's height.
    pub height: Sizing,

    /// The selected @form.choice.value[`values`] of this field,
    /// but represented as indexes of @form.choice.options[`options`].
    #[internal]
    #[synthesized]
    pub value_indexes: EcoVec<usize>,
}

impl ShowSet for Packed<FormChoice> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the field itself available to the `FieldWidget`. Should be
        // superseded by a proper ancestry mechanism in the future.
        let mut elem = self.clone().pack();
        elem.materialize(styles);
        out.set(FormWidget::parent, Some(elem));

        let disabled = self.disabled.get(styles);
        styles::set_rect_stroke(&mut out, disabled);
        if disabled {
            styles::set_disabled_styles(&mut out);
        }
        out
    }
}

impl Synthesize for Packed<FormChoice> {
    fn synthesize(&mut self, _: &mut Engine, styles: StyleChain) -> SourceResult<()> {
        let span = self.span();

        let mut seen_options = FxHashMap::default();
        for (i, option) in self.options.get_ref(styles).0.iter().enumerate() {
            if seen_options.insert(option.mapping_name.as_str(), i).is_some() {
                bail!(
                    span, "duplicate option";
                    hint: "occurred in option at index {i} (`{}`)", option.mapping_name.repr();
                )
            }
        }

        let value = &self.value.get_ref(styles).0;

        if !self.multiple.get(styles) && value.len() > 1 {
            bail!(
                span, "field cannot have more than one value when `multiple` is `false`";
                hint: "set `multiple` to `true` or set only a single value for this field";
            )
        }
        self.value_indexes = Some(value.iter().map(|v| {
            seen_options.get(v.as_str()).copied().ok_or_else(||{
                eco_vec![error!(
                    span, "unknown value `{}`", v.repr();
                    hint: "value is not one of the options of this field";
                    hint: "try adding it to the `options` property or setting a different value instead";
                )]
            })
        }).collect_combined_result()?);

        Ok(())
    }
}

/// Available options in a choice form field.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct ChoiceOptions(pub EcoVec<ChoiceOption>);

cast! {
    ChoiceOptions,
    self => self.0
        .into_iter()
        .map(|option| {
            (option.mapping_name.into(), option.display_name.into_value())
        })
        .collect::<Dict>()
        .into_value(),
    values: Array => Self(values
        .into_iter()
        .enumerate()
        .map(|(i, v)| Ok(ChoiceOption::new(
            v.clone().cast::<EcoString>().hint(opt_hint_helper(i, &v))?,
            Smart::Auto,
        )))
        .collect::<HintedStrResult<_>>()?),
    values: Dict => Self(values
        .into_iter()
        .enumerate()
        .map(|(i, (k, v))| Ok(ChoiceOption::new(
            k.clone().into_value().cast::<EcoString>().hint(opt_hint_helper(i, &k))?,
            v.cast::<Smart<EcoString>>().hint(opt_hint_helper(i, &k))?,
        )))
        .collect::<HintedStrResult<_>>()?),
}

fn opt_hint_helper(index: usize, key: &impl Repr) -> EcoString {
    eco_format!("occurred in option at index {index} (`{}`)", key.repr())
}

/// An option in a choice form field.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct ChoiceOption {
    /// The name that uniquely identifies this option.
    pub mapping_name: EcoString,
    /// The user-facing name of this option.
    /// If [`Smart::Auto`], [`Self::mapping_name`] is used instead.
    pub display_name: Smart<EcoString>,
}

impl ChoiceOption {
    pub fn new(mapping_name: EcoString, display_name: Smart<EcoString>) -> Self {
        Self { mapping_name, display_name }
    }

    /// If present, get the display name, otherwise get the mapping name.
    pub fn display_or_mapping_name(&self) -> EcoString {
        self.display_name.as_ref().unwrap_or(&self.mapping_name).clone()
    }
}

/// TODO
#[elem(name = "signature", since = "unreleased", Locatable, Tagged, ShowSet)]
pub struct FormSignature {
    /// The name of this field.
    ///
    /// If `{auto}`, will be automatically generated as `field-<n>`
    /// when exporting to PDF.
    pub name: Smart<EcoString>,

    /// The field's width.
    pub width: Smart<Rel<Length>>,

    /// The field's height.
    pub height: Sizing,
}

impl ShowSet for Packed<FormSignature> {
    fn show_set(&self, styles: StyleChain) -> Styles {
        let mut out = Styles::new();
        // Makes the field itself available to the `FieldWidget`. Should be
        // superseded by a proper ancestry mechanism in the future.
        let mut elem = self.clone().pack();
        elem.materialize(styles);
        out.set(FormWidget::parent, Some(elem));

        styles::set_rect_stroke(&mut out, false);
        out
    }
}

/// A label describing a form field.
///
/// In order to create an accessible form, all fields must
/// have an accompanying label indicating what they represent.
/// While a label can be placed anywhere in the document,
/// it is recommended for it to be close to the field it describes.
///
/// Furthermore, when exporting to HTML, labels also act as focus target
/// for their linked field.
/// For example, clicking the label of a checkbox will toggle that checkbox,
/// and clicking the label of a textbox will focus the cursor in that textbox.
#[elem(name = "label", since = "unreleased", Locatable)]
pub struct FormLabel {
    /// The form field that this label describes.
    #[required]
    pub target: Label,

    /// The description of the linked form field.
    #[required]
    pub body: Content,
}

impl FormLabel {
    /// Resolves the destination.
    pub fn resolve_early(
        &self,
        engine: &mut Engine,
        span: Span,
    ) -> SourceResult<Location> {
        let elem = engine
            .introspect(QueryLabelIntrospection(self.target, span))
            .at(span)?;
        if !labelable_element_selector().matches(&elem, None) {
            bail!(span, "label `{}` does not point to a form field", self.target.repr())
        }
        Ok(elem.location().unwrap())
    }

    /// Resolves the destination without an engine.
    pub fn resolve_late(&self, introspector: &dyn Introspector) -> StrResult<Location> {
        let elem = introspector.query_label(self.target)?;
        Ok(elem.location().unwrap())
    }

    /// Finds all linked-to inputs referenced in an introspector.
    pub fn find_destinations(
        introspector: &dyn Introspector,
    ) -> impl Iterator<Item = Location> {
        introspector
            .query(&Self::ELEM.select())
            .into_iter()
            .map(|elem| elem.into_packed::<Self>().unwrap())
            .filter_map(|elem| elem.resolve_late(introspector).ok())
    }
}

/// Check if the given element is inside a form field;
/// if so, returns an error.
pub fn assert_not_inside_form_field<T: NativeElement>(
    elem: &Packed<T>,
    styles: StyleChain,
) -> SourceResult<()> {
    if let Some(parent) = styles
        .get_all_ref(FormWidget::parent)
        .filter_map(|opt| opt.as_ref())
        .find(|el| el.location() != elem.location())
    {
        bail!(
            elem.span(),
            "this {} element cannot be inside a form field", T::ELEM.name();
            hint[parent.span()]: "the outer form field is here";
        )
    }
    Ok(())
}

/// Generate an implicit field name based on a counter value.
pub fn gen_field_name(counter: u64) -> EcoString {
    eco_format!("field-{}", counter)
}

/// The representation of a form field in the document.
#[derive(Debug, Clone, Hash)]
pub struct FieldWidget {
    /// The span of the element declaring this widget.
    pub span: Span,
    /// The location of field this widget belongs to.
    /// If this widget is a radio button, this points to the radio group.
    pub field_location: Location,
    /// The size of the widget.
    pub size: Size,
    /// The action to perform when clicking this widget, if any.
    pub action: Option<WidgetAction>,
    /// The value this widget represents, if any. Used for radio buttons.
    pub value: Option<EcoString>,
    /// The appearance of this widget.
    pub appearance: Arc<FieldAppearanceKind<Frame>>,
}

/// The appearance of a field widget.
///
/// Contains a body of a generic type `T`, so that it can be reused with different types.
///
/// For checkboxes and radio buttons, the appearance has two bodies instead of just one,
/// corresponding to their "off" and "on" states.
/// Each body may also have different interactive states; see [`FieldAppearanceBody`].
#[derive(Debug, Clone, Hash)]
pub enum FieldAppearanceKind<T> {
    /// A static appearance (e.g., button).
    Single { variable_text: Option<VariableText>, body: FieldAppearanceBody<T> },
    /// A dual state appearance (e.g., checkbox).
    Dual {
        /// Whether this checkbox/radio is checked.
        active: bool,
        /// The body when
        body_off: FieldAppearanceBody<T>,
        body_on: FieldAppearanceBody<T>,
    },
}

impl<T> FieldAppearanceKind<T> {
    /// Get the primary body of this appearance.
    ///
    /// For single appearances, returns the only body.
    /// For dual appearances, returns the off body.
    pub fn primary_body(&self) -> &FieldAppearanceBody<T> {
        match self {
            Self::Single { body, .. } => body,
            Self::Dual { body_off, .. } => body_off,
        }
    }

    /// Get the active body of this appearance.
    pub fn active_body(&self) -> &FieldAppearanceBody<T> {
        match self {
            Self::Single { body, .. } => body,
            Self::Dual { active, body_on, .. } if *active => body_on,
            Self::Dual { body_off, .. } => body_off,
        }
    }

    /// Whether this appearance is of type 'variable text'.
    pub fn variable_text(&self) -> Option<&VariableText> {
        match self {
            Self::Single { variable_text, .. } => variable_text.as_ref(),
            Self::Dual { .. } => None,
        }
    }

    /// If this is of variant [`Self::Single`], convert to [`Option<T>`].
    pub fn single(self) -> Option<FieldAppearanceBody<T>> {
        match self {
            Self::Single { body, .. } => Some(body),
            Self::Dual { .. } => None,
        }
    }

    /// If this is of variant [`Self::Dual`], convert to [`Option<(T, T)>`],
    /// that is an [`Option`] of `(body_off, body_on)`.
    pub fn dual(self) -> Option<(FieldAppearanceBody<T>, FieldAppearanceBody<T>)> {
        match self {
            Self::Single { .. } => None,
            Self::Dual { body_off, body_on, .. } => Some((body_off, body_on)),
        }
    }

    /// Maps this [`FieldAppearanceKind`] to another body type by applying a
    /// function to its body/bodies.
    pub fn try_map<N, E>(
        &self,
        mut f: impl FnMut(&T) -> Result<N, E>,
    ) -> Result<FieldAppearanceKind<N>, E> {
        match self {
            Self::Single { variable_text, body } => Ok(FieldAppearanceKind::Single {
                variable_text: variable_text.clone(),
                body: body.try_map(&mut f)?,
            }),
            Self::Dual { active, body_off, body_on } => Ok(FieldAppearanceKind::Dual {
                active: *active,
                body_off: body_off.try_map(&mut f)?,
                body_on: body_on.try_map(&mut f)?,
            }),
        }
    }
}

/// The body of a single state of a field widget.
///
/// May contain additional representations for the hover and down interactive states.
#[derive(Debug, Clone, Hash)]
pub struct FieldAppearanceBody<T> {
    /// The appearance in the normal state, where the widget is not being interacted with.
    pub normal: T,
    /// The appearance when the user's cursor is hovering over the widget.
    pub hover: Option<T>,
    /// The appearance when the user is clicking on the widget.
    pub down: Option<T>,
}

impl<T> FieldAppearanceBody<T> {
    /// Create a new [`FieldAppearanceBody`] with the given normal state,
    /// and [`None`] for hover and down state.
    pub fn new(normal: T) -> Self {
        FieldAppearanceBody { normal, hover: None, down: None }
    }

    /// Maps this [`FieldAppearanceBody`] to another body type by applying a
    /// function to its states.
    fn try_map<N, E>(
        &self,
        mut f: impl FnMut(&T) -> Result<N, E>,
    ) -> Result<FieldAppearanceBody<N>, E> {
        Ok(FieldAppearanceBody {
            normal: f(&self.normal)?,
            hover: self.hover.as_ref().map(&mut f).transpose()?,
            down: self.down.as_ref().map(&mut f).transpose()?,
        })
    }
}

/// Styles to apply to variable text.
#[derive(Debug, Clone, Hash)]
pub struct VariableText {
    /// The type of font to use (sans-serif, serif, or monospace).
    pub font_type: FontType,
    /// Whether to use the bold variant of the font.
    pub font_bold: bool,
    /// Whether to use the italic variant of the font.
    pub font_italic: bool,
    /// Resolved font size.
    pub font_size: Abs,
    /// Text color. Only solid colors are supported.
    pub fill_color: Paint,
}

/// The type of the font.
#[derive(Debug, Copy, Clone, Hash)]
pub enum FontType {
    /// Sans serif.
    Sans,
    /// Serif.
    Serif,
    /// Monospace.
    Mono,
}

/// Action to perform when a widget is clicked on.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum WidgetAction {
    /// Submit all fields in a form to a given URL.
    Submit { form: Location, target: FormTarget },
    /// Reset all fields in the form to their default value.
    Reset { form: Location },
}

/// Get a selector that targets all form field types.
pub fn form_fields_selector() -> &'static Selector {
    singleton!(
        Selector,
        Selector::Or(eco_vec![
            FormButton::ELEM.select(),
            FormCheckbox::ELEM.select(),
            FormRadioGroup::ELEM.select(),
            FormTextbox::ELEM.select(),
            FormChoice::ELEM.select(),
        ])
    )
}

/// Get a selector that targets all elements that can be labeled via [`FormLabel`].
fn labelable_element_selector() -> &'static Selector {
    singleton!(
        Selector,
        Selector::Or(eco_vec![
            FormButton::ELEM.select(),
            FormCheckbox::ELEM.select(),
            FormRadioGroup::ELEM.select(),
            FormRadioButton::ELEM.select(),
            FormTextbox::ELEM.select(),
            FormChoice::ELEM.select(),
        ])
    )
}

/// Fragment element to allow styling of form fields in a specific state.
///
/// This element cannot be constructed manually and is part of the default
/// show rule for each form field.
///
/// Further information can be found in the
/// @guides:forms:styling[Styling section of the Forms Guide].
#[elem(name = "widget", since = "unreleased", Construct)]
pub struct FormWidget {
    /// Whether the form field is checked.
    ///
    /// This is `{none}` if the form field does not have a checked state.
    /// Only @form.checkbox[checkboxes] and @form.radio[radio buttons] have checked
    /// states.
    pub checked: Option<bool>,
    /// The interactive state the form field is in.
    ///
    /// This is `{none}` if the form field only has the normal state.
    pub state: Option<WidgetState>,

    /// The form field this widget belongs to.
    /// This is a bit of a hack and should be superseded by a proper ancestry mechanism.
    #[ghost]
    #[internal]
    pub parent: Option<Content>,
}

impl Construct for FormWidget {
    fn construct(_: &mut Engine, args: &mut Args) -> SourceResult<Content> {
        bail!(args.span, "cannot be constructed manually")
    }
}

/// The interactive state the widget represents.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Cast)]
pub enum WidgetState {
    /// The normal state, where the widget is not being interacted with.
    Normal,
    /// The user's cursor is hovering over the widget.
    Hovered,
    /// The user is clicking on the widget.
    Down,
}

/// The writing area of a @form.textbox[textbox].
///
/// A cursor element can only be used inside the show rule of
/// a textbox.
///
/// Further information can be found in the
/// @guides:forms:textbox[Styling Textboxes section of the Forms Guide].
#[elem(name = "cursor", since = "unreleased")]
pub struct FormCursor {}

/// Constants and helper functions to style form fields.
pub mod styles {
    use crate::foundations::{Smart, Styles};
    use crate::layout::{Abs, Sides};
    use crate::text::TextElem;
    use crate::visualize::{Color, Luma, Paint, ProcessColor, RectElem, Rgb, Stroke};

    /// The thickness of the default stroke of form fields, in points.
    pub const STROKE_THICKNESS_PT: f64 = 0.5;

    /// Background color of disabled field.
    pub(super) const DISABLED_FILL: Color =
        Color::Process(ProcessColor::Luma(Luma::new(0.95, 0.7)));
    /// Stroke color of disabled field.
    const DISABLED_STROKE_COLOR: Color =
        Color::Process(ProcessColor::Luma(Luma::new(0.0, 0.5)));
    /// Text color of disabled field.
    pub const DISABLED_TEXT_FILL: Color =
        Color::Process(ProcessColor::Luma(Luma::new(0.15, 0.7)));

    /// Background color of a selected option of a listbox.
    pub const LISTBOX_SELECTED_FILL: Color =
        Color::Process(ProcessColor::Rgb(Rgb::new(0.0, 0.47843137, 1.0, 1.0)));
    /// Background color of a selected option of a disabled listbox.
    pub const LISTBOX_SELECTED_DISABLED_FILL: Color =
        Color::Process(ProcessColor::Luma(Luma::new(0.15, 0.21)));

    /// Get the stroke for a field, depending whether it is disabled.
    pub(super) fn field_stroke(disabled: bool) -> Stroke {
        if disabled {
            Stroke::from_pair(DISABLED_STROKE_COLOR, Abs::pt(STROKE_THICKNESS_PT).into())
        } else {
            Stroke::from_thickness(Abs::pt(STROKE_THICKNESS_PT))
        }
    }

    /// Get the background of a selected option of a listbox,
    /// depending whether the listbox is disabled.
    pub fn listbox_selected_paint(disabled: bool) -> Paint {
        if disabled {
            LISTBOX_SELECTED_DISABLED_FILL.into()
        } else {
            LISTBOX_SELECTED_FILL.into()
        }
    }

    /// Set the background and text color for a disabled field.
    pub(super) fn set_disabled_styles(styles: &mut Styles) {
        styles.set(RectElem::fill, Some(DISABLED_FILL.into()));
        styles.set(TextElem::fill, DISABLED_TEXT_FILL.into());
    }

    /// Set the stroke and accompanying outset/inset for a form field
    /// that is surrounded by a [`RectElem`].
    pub(super) fn set_rect_stroke(styles: &mut Styles, disabled: bool) {
        styles.set(
            RectElem::outset,
            Sides::splat(Some(Abs::pt(-STROKE_THICKNESS_PT / 2.0).into())),
        );
        styles.set(
            RectElem::stroke,
            Smart::Custom(Sides::splat(Some(Some(field_stroke(disabled))))),
        );
        styles.set(
            RectElem::inset,
            Sides::splat(Some(Abs::pt(STROKE_THICKNESS_PT).into())),
        );
    }
}
