// Test interactive forms.

--- form-basic paged html pdftags ---
#set pdf(standard: "ua-1")

// One of each field type.

#form[
  #form.button(action: "reset")[Reset button]

  #form.label(<checkbox>)[Checkbox]
  #form.checkbox() <checkbox>

  #form.radio-group(title: [A radio group])[
    #form.label(<radio>)[Radio]
    #form.radio() <radio>
  ]

  #form.label(<textbox>)[Textbox]
  #form.textbox(width: 100pt) <textbox>

  #form.label(<textarea>)[Textarea]
  #form.textbox(multiline: true) <textarea>

  #form.label(<dropdown>)[Dropdown]
  #form.choice(options: ("foo", "bar")) <dropdown>

  #form.label(<listbox>)[Listbox]
  #form.choice(options: ("foo", "bar"), multiple: true) <listbox>
]

--- form-target-invalid-content-type-html html ---
// HTML cannot export to FDF, XFDF, nor PDF.
// Error: 2-67 content-type `"fdf"` is not supported in HTML export
#form(target: (url: "https://example.com", content-type: "fdf"))[]
// Error: 2-68 content-type `"xfdf"` is not supported in HTML export
#form(target: (url: "https://example.com", content-type: "xfdf"))[]
// Error: 2-67 content-type `"pdf"` is not supported in HTML export
#form(target: (url: "https://example.com", content-type: "pdf"))[]

--- form-target-invalid-content-type-pdf pdf ---
// PDF cannot export to multipart nor plain text.
// Error is only triggered when there is a submit button.

#form(target: (url: "https://example.com", content-type: "multipart"))[
  // Error: 4-35 content-type `"multipart"` is not supported in PDF export
  #form.button(action: "submit")[]
]

#form(target: (url: "https://example.com", content-type: "plain"))[
  // Error: 4-35 content-type `"plain"` is not supported in PDF export
  #form.button(action: "submit")[]
]

--- form-target-warning-content-type-pdf pdf ---
// PDF readers have poor support for all formats that are not FDF.
// Warning is only triggered when there is a submit button.

#form(target: (url: "https://example.com", content-type: "url-encoded"))[
  // Warning: 4-41 content-type `"url-encoded"` has poor support among PDF readers
  // Hint: 4-41 prefer the `"fdf"` content-type whenever possible
  #form.button(action: "submit")[Submit]
]

#form(target: (url: "https://example.com", content-type: "xfdf"))[
  // Warning: 4-41 content-type `"xfdf"` has poor support among PDF readers
  // Hint: 4-41 prefer the `"fdf"` content-type whenever possible
  #form.button(action: "submit")[Submit]
]

#form(target: (url: "https://example.com", content-type: "pdf"))[
  // Warning: 4-41 content-type `"pdf"` has poor support among PDF readers
  // Hint: 4-41 prefer the `"fdf"` content-type whenever possible
  #form.button(action: "submit")[Submit]
]

--- form-target-method-get-pdf pdf ---
// PDF cannot use GET method.
// Error is only triggered when there is a submit button.
#form(target: (url: "https://example.com", method: "get"))[
  // Error: 4-35 method `"GET"` is not supported in PDF export
  #form.button(action: "submit")[]
]

--- form-label-across-documents bundle ---
// HTML labels can only target fields in the same document.
#document("index.html")[
  // Error: 4-38 form labels can only target fields in the same document
  #form.label(<checkbox>)[A checkbox]
]
#document("page.html")[
  #form[
    #form.checkbox() <checkbox>
  ]
]

--- form-label-not-field pdf html ---
#form[
  = Heading <heading>

  // Error: 4-32 label `<heading>` does not point to a form field
  #form.label(<heading>)[Label]
]

--- form-submit-button-without-target paged ---
// Hint: 1:2-3:2 set the `target` property of the surrounding `#form` element
#form[
  // Error: 4-41 button has a submit action, but no submission target is defined for its form
  #form.button(action: "submit")[Submit]
]

--- form-cursor-outside-textbox paged ---
// Error: 2-15 this cursor element must be inside a textbox element
// Hint: 2-15 you can use it in a show rule such as ```\n#show form.textbox: {\n   show form.widget: form.cursor()\n   it\n}\n```
#form.cursor()

--- form-textbox-full-width-errors paged ---
#form[
  // Error: 4-52 `fixed-width` cannot be `true` for multiline textboxes
  #form.textbox(multiline: true, fixed-width: true)
  // Error: 4-35 `max-length` must be set when `fixed-width` is `true`
  #form.textbox(fixed-width: true)
]

--- form-radio-group-selected-not-button pdf html ---
#form[
  == Heading <heading>

  // Error: 1:4-4:4 label `<heading>` does not exist in the radio group
  #form.radio-group(selected: <heading>)[
    #form.radio() <radio-1>
    #form.radio() <radio-2>
  ]
]

--- form-radio-group-selected-multiple pdf html ---
#form[
  // Error: 1:4-4:4 label `<radio-1>` occurs multiple times in the radio group
  #form.radio-group(selected: <radio-1>)[
    #form.radio() <radio-1>
    #form.radio() <radio-1>
  ]
]

--- form-radio-group-selected-outside-group pdf html ---
#form[
  #form.radio-group()[
    #form.radio() <radio-1>
    #form.radio() <radio-2>
  ]

  // Error: 1:4-4:4 label `<radio-2>` does not exist in the radio group
  #form.radio-group(selected: <radio-2>)[
    #form.radio() <radio-3>
    #form.radio() <radio-4>
  ]
]

--- form-choice-duplicate-options paged html ---
#form[
  // Error: 1:4-3:4 duplicate option
  // Hint: 1:4-3:4 occurred in option at index 2 (`"foo"`)
  #form.choice(
    options: ("foo", "bar", "foo", "baz"),
  )
]

--- form-choice-multiple-values-on-dropdown paged html ---
#form[
  // Error: 1:4-4:4 field cannot have more than one value when `multiple` is `false`
  // Hint: 1:4-4:4 set `multiple` to `true` or set only a single value for this field
  #form.choice(
    options: ("foo", "bar", "baz"),
    value: ("bar", "baz"),
  )
]

--- form-choice-unknown-value paged html ---
#form[
  // Error: 1:4-4:4 unknown value `"Foo"`
  // Hint: 1:4-4:4 value is not one of the options of this field
  // Hint: 1:4-4:4 try adding it to the `options` property or setting a different value instead
  #form.choice(
    options: ("foo": "Foo"),
    value: "Foo",
  )
]

--- form-widget-cannot-be-contructed eval ---
// Error: 2-15 cannot be constructed manually
#form.widget()

--- form-pdf-2_0-standard-fonts pdf ---
// FIXME: this test should stop throwing an error whenever this is fixed.
// Error: a PDF standard font was used, but that is not allowed in PDF 2.0
// Hint: these fonts are used by textboxes and choice fields
// Hint: set the PDF version to PDF 1.7 or earlier, or remove form fields
#set pdf(standard: "2.0")

#form[
  #form.textbox()
]

--- form-variable-text-non-solid-color pdf ---
#let color = gradient.linear(red, green)

#form[
  #show form.textbox: it => {
    show form.cursor: set text(fill: color)
    it
  }
  // Error: 4-18 text of a form field can only be filled with a solid color
  // Hint: 4-18 remove the gradient or pattern
  #form.textbox()

  #show form.choice: it => {
    set text(fill: color)
    it
  }

  // Error: 4-17 text of a form field can only be filled with a solid color
  // Hint: 4-17 remove the gradient or pattern
  #form.choice()
  // Error: 4-31 text of a form field can only be filled with a solid color
  // Hint: 4-31 remove the gradient or pattern
  #form.choice(multiple: true)
]

--- form-duplicate-field-names pdf ---
#form[
  // Hint: 4-30 the other field is here
  #form.checkbox(name: "foo")
  // Error: 4-30 two fields share the same name ("foo")
  // Hint: 4-30 change the name of one of the fields
  #form.checkbox(name: "foo")
]

--- form-duplicate-fully-qualified-field-names pdf ---
#form[
  // Error: 4-34 there are two distinct form fields named `foo.bar`
  // Hint: 4-34 this can happen if you have a field named `foo.bar` and another named `foo.bar.something-else`
  #form.checkbox(name: "foo.bar")
  #form.checkbox(name: "foo.bar.baz")
]

--- form-field-outside-form paged html ---
// Error: 2-15 this button element must be placed inside a form element
// Hint: 2-15 try surrounding this element with `#form[...]`
#form.button[]
// Error: 2-17 this checkbox element must be placed inside a form element
// Hint: 2-17 try surrounding this element with `#form[...]`
#form.checkbox()
// Error: 2-20 this radio-group element must be placed inside a form element
// Hint: 2-20 try surrounding this element with `#form[...]`
#form.radio-group[]
// Error: 2-16 this textbox element must be placed inside a form element
// Hint: 2-16 try surrounding this element with `#form[...]`
#form.textbox()
// Error: 2-15 this choice element must be placed inside a form element
// Hint: 2-15 try surrounding this element with `#form[...]`
#form.choice()

--- form-radio-button-outside-radio-group paged html ---
#form[
  // Error: 4-16 this radio button must be placed inside a radio group
  // Hint: 4-16 try surrounding this button with `#form.radio-group[...]`
  #form.radio()
]

--- form-missing-field-alt-name-pdf-ua pdf ---
#set pdf(standard: "ua-1")

#form[
  // Error: 4-18 PDF/UA-1 error: missing alt name
  // Hint: 4-18 make sure your form fields have a label
  // Hint: 4-18 add a `#form.label` that points to this field
  // Hint: 4-18 for `#form.radio-group`, you can set the `title` property instead
  #form.textbox()
]

--- form-hover-down-styles-pdf-a pdf ---
// Error: PDF/A-3a error: missing document date
// Hint: set the date of the document
#set pdf(standard: "a-3a")

#form[
  #show form.checkbox: it => {
    show form.widget.where(state: "hovered"): set rect(stroke: red)
    it
  }
  #form.label(<checkbox-1>)[Checkbox]
  // Error: 4-19 PDF/A-3a error: annotation has hover or down styles
  // Hint: 4-19 hover and down styles are not supported in this PDF standard
  // Hint: 4-19 remove `show form.widget.where(state: ...)` rules
  #form.checkbox() <checkbox-1>
]

#form[
  #show form.checkbox: it => {
    show form.widget.where(state: "down"): set rect(stroke: yellow)
    it
  }
  #form.label(<checkbox-2>)[Checkbox]
  // Error: 4-19 PDF/A-3a error: annotation has hover or down styles
  // Hint: 4-19 hover and down styles are not supported in this PDF standard
  // Hint: 4-19 remove `show form.widget.where(state: ...)` rules
  #form.checkbox() <checkbox-2>
]

--- form-textbox-fixed-width-html html ---
#form[
  // Warning: 4-35 `fixed-width` property was ignored during HTML export
  // Hint: 4-35 the property is not supported in this export format
  #form.textbox(fixed-width: true)
]

--- form-textbox-placeholder-paged paged ---
#form[
  // Warning: 4-53 ignored `placeholder` property when exporting
  // Hint: 4-53 property not supported in this export format
  #form.textbox(placeholder: "foobar", width: 100pt)
]
