#import "../../components/index.typ": docs-chapter, docs-table, short-or-long

#show: docs-chapter.with(
  title: "Form Guide",
  route: "/guides/forms",
  description: "Not sure how to give your checkboxes their own style? Or how to allow your users to submit your form? This guide explains all you need to know about interactive forms in Typst.",
)

#let todo = it => html.elem("span", attrs: (style: "background: red; color: white;"), strong[TODO: #it])

With interactive forms, you can create PDFs and web pages that can be filled out by your users.
Forms can be used for a wide set of applications, ranging from collecting personal data
in a business setting to creating fillable attendance sheets at school. #todo[better examples?]
Typst provides you with the tools to create the form that better suits your needs,
no matter if you plan to distribute it digitally via PDF, on a web page, or even printed.

When distributed via PDF, your users can open the file in their PDF reader, fill out the form,
save the file, and then send it back to you.
Most modern PDF readers support interactive forms, including the built-in readers of all
major web browsers.
Certain readers also support features such as submitting the filled out data directly from
the PDF.
Check the @form:compatibility[compatibility table] in the forms reference page for a detailed
overview of reader support.

Furthermore, when distributed via a web page, you can enable users to submit the form
directly on your website, without having to share files out-of-band.
Typst's forms can be natively exported to HTML in addition to PDF,
and work out-of-the-box for most use cases.
Whenever you need more flexibility,
you can always use explicit HTML elements via the @html module.

This guide takes you through a few of the most common questions you may have
when adding an interactive form to your document with Typst.
Feel free to skip to the section most relevant to you – we designed this guide to be read out of order.

If you want to look up a detail of how forms or form fields work,
you should also @form[check out their reference page].

= #short-or-long[Form Fields][Which form fields does Typst support?] <form-fields>

Typst has support for five kinds of form fields where users can input information,
plus an actionable push button.
Each field kind is better suited for different data that you might want to collect
and the format you want to distribute your form in.

- *Checkbox:* select between two options, for example "yes" and "no".
  Checkboxes are suited for when a user needs to perform a binary choice.
  They work great digitally and in print.
  ```preview
  #form[
    #set align(center)
    #grid(
      columns: 2,
      gutter: 2em,
      form.checkbox(),
      form.checkbox(checked: true),
    )
  ]
  ```

- *Radio:* select at most one option among multiple options.
  Radio groups and buttons are suited for when the user needs to make an exclusive choice.
  They work great digitally, but do not prevent users from selecting multiple options in print.
  ```preview
  #form[
    #show form.radio: box.with(baseline: 0.2em)
    #set align(center + horizon)
    #show grid.cell: set align(horizon)
    #form.radio-group(selected: <blue>)[
      #grid(
        columns: (1fr, 1fr, 1fr),
        gutter: 2em,
        [#form.radio() Red],
        [#form.radio() Green],
        [#form.radio() <blue> Blue],
      )
    ]
  ]
  ```

- *Textbox:* write free-form text.
  Textboxes are ideal for when you need the user to provide arbitrary text,
  no matter the length.
  Typst supports both single-line and multi-line textboxes, as well as
  a fixed-width variant exclusive to PDF that is ideal for structured
  data such as ID numbers.
  Textboxes work great both digitally and in print.
  #todo[fixed-width will get new default styling]
  ```preview
  #form[
    #set form.textbox(width: 100%)
    #form.label(<name>)[Name:]
    #form.textbox() <name>

    #form.label(<about>)[About:]
    #form.textbox(multiline: true) <about>

    #form.label(<id>)[ID number:]
    #form.textbox(
      fixed-width: true,
      max-length: 9,
    ) <id>
  ]
  ```

- *Choice:* select one or more options from a list.
  When only a single choice is allowed, this field kind is displayed as
  a dropdown. For multiple choices, it is displayed as a listbox instead.
  Choice fields work great digitally, but they are unusable in print.
  Consider using checkboxes or radio buttons instead if you need to print your form.
  ```preview
  #let options = ("Red", "Green", "Blue")
  #set form.choice(options: options, width: 100%)
  #form[
    #grid(
      columns: 2,
      gutter: 2em,
      [
        Dropdown:
        #form.choice(value: "Blue")
      ],
      [
        Listbox:
        #form.choice(multiple: true, value: ("Green", "Blue"))
      ],
    )
  ]
  ```

- *Signature:* digitally sign the document and the form.
  Signature fields use cryptography to create a signature of the contents of
  the document, attesting its validity.
  Users of a supported PDF reader can sign the document using their digital signature.
  Only certain PDF readers are supported - check the
  @form:compatibility[compatibility table] for full details.
  Signature fields are not supported in HTML,
  but are supported in print for handwritten signatures.
  #todo[example when implemented]

- *Button:* perform an action when clicked.
  While not a field that can record information,
  it can be used for submitting or resetting a form when clicked.
  When distributing your form via a web page, you can use custom JavaScript to
  perform a custom action instead.
  Button's actions can only be performed digitally, never in print.
  ```preview
  #set align(center)
  #form[
    #grid(
      columns: 2,
      gutter: 2em,
      form.button[Submit],
      form.button[Reset],
    )
  ]
  ```

You can mix and match field kinds inside the same form
according to your needs.

= #short-or-long[Basic Forms][How to create a basic form?] <basic-forms>
In order to create a form in Typst, use the @form[`form` function].
Next, you need to add fields to your form.
For this example, let's say we want to ask the user their name.

```example
#form[
  #form.label(<name>)[What is your name?]
  #form.textbox() <name>
]
```

This example shows how to create a form with a single field and label.
The only required argument of the `form` function is the form's content,
which is passed as a @content[content block] using square brackets.

The first element in the form's content is a @form.label.
This element is different and serves a different purpose
than the @label[`label` function] and its dedicated angle-brackets syntax (`{<name>}`).
While @label is used to reference elements in Typst,
@form.label is a visible description of a form field
that is also used to provide an alternative description for
users of Assistive Technology.
While not strictly required, we recommend having a @form.label for every form field
in your document.
The @form.label[`form.label` function] takes two required arguments: a @label to
the form field it describes, and the description of that form field.
In this example, the former is the label `{<name>}`
and the latter is the content block `{[What is your name?]}`.

The second element in the form's content is a @form.textbox.
This is a form field that accepts free-form text,
which is ideal for our example since the user can fill out their name there.
The @form.textbox[`form.textbox` function] has no required arguments.
The textbox has the `{<name>}` @label attached to it,
so that it can be described by the @form.label that precedes it.

After having created your first form, you can now explore other kinds of fields
such as @form.checkbox[checkboxes], @form.radio-group[radio groups],
@form.choice[choice fields], and @form.signature[signature fields].

= #short-or-long[Positioning][How to place fields next to text?] <positioning>

All form fields are block-level elements and cannot occur inside of a @par[paragraph].
To change this behavior, we can use the @box[`box` function] to integrate form fields
into a paragraph:

```example
#form[
  #box(form.checkbox())
  This text is next to the checkbox
]
```

In practice, the `box` turns the checkbox into an inline element,
and places it next to the subsequent text.
However, having to wrap every form field in a @box can get burdensome
when you have a dozen or more form fields.
To ease this, we can use a show rule for the checkbox:

```example
#show form.checkbox: box

#form[
  #form.checkbox()
  This text is next to the checkbox

  #form.checkbox()
  So is this one
]
```

With this show rule, all checkboxes in the document will be placed inline!

#todo[baseline? perhaps we should fix it by default]

There are also other alternatives for placing form fields next to text.
For instance, you can use the @grid[`grid` function] to align checkboxes
and their labels in a 2-column grid:

```example
= Room Booking

#form[
  Which periods would you like to book the room for?

  #grid(
    columns: 2,
    gutter: 0.4em,
    align: horizon,
    [#form.checkbox() <morning>],   form.label(<morning>)[Morning],
    [#form.checkbox() <afternoon>], form.label(<afternoon>)[Afternoon],
    [#form.checkbox() <evening>],   form.label(<evening>)[Evening],
  )
]
```

This example shows how to create a form with three checkboxes, their labels,
and surrounding text, horizontally aligning the checkboxes with their respective labels
via a grid.

Here, we pass three named arguments to the @grid function call,
setting the number of columns to `{2}`,
the spacing between each grid cell to `{0.4em}`,
and to @alignment[horizontally align] each one in the middle.
Then, we pass positional arguments with both checkboxes and @form.label[form labels].
Since it is not possible to attach a @label to an element in code mode,
we must change to markup mode via square brackets `[]` when passing the checkboxes as arguments,
e.g. `[#form.checkbox() <morning>]`.
This is not needed for @form.label, so those arguments are passed in code mode,
omitting the `#` character.

To make the example more legible, we have placed a checkbox and @form.label arguments on each line,
mimicking how they would appear in the grid.
You could also write each cell on its own line.
Typst does not care on which line you place the arguments.
Instead, Typst will place the content cells from left to right
(or right to left, if that is the writing direction of your language) and then from top to bottom.
It will automatically add enough rows to your grid so that it fits all of your content.

= #short-or-long[Styling][How to change the appearance of form fields?] <styling>

While Typst provides a default appearance for each field type,
you are free to customize them as you see fit through @reference:styling:show-rules[show rules]
as explained in this section.

#todo[
  I made this up, not sure if pseudo-element is a good name, but I want to highlight that they are slightly special.
  Name comes from HTML's pseudo selectors.
  This whole section probably needs further rewriting to properly convey its message.
  A diagram or something like that may help, but I'm hesitant in showing typst code that doesn't really compile
]
Given their interactive nature, all fields exclusively contain one or more
@form.widget[`form.widget` pseudo-elements].
We call them pseudo-elements because, unlike regular elements, they are not
always visible in the document and instead represent a single state the field can be in.
For instance, a @form.checkbox's default show rule contains a total of 6 widgets:
the checkbox can either be unchecked or checked, and at the same time it can be unfocused, hovered, or being clicked on,
resulting in 6 different combinations.

When opening a PDF with interactive form fields, only one of these widgets is shown per field at a given time.
The widget that is currently visible can change when the field is interacted with.
For example, when an unchecked checkbox is clicked, it becomes checked and its `{form.widget(checked: false)}` is hidden
and its `{form.widget(checked: true)}` is shown.

The existence of these pseudo-elements allows the various states of each field to be
styled via show rules, even if those states are not active when the document is created.
For example, to write a show rule for the checked and hovered state of a checkbox,
you can create a nested show rule as shown:

```typ
#show form.checkbox: it => {
  show form.widget.where(checked: true, state: "hovered"): ... // your code here
  it
}
```

In the next sub-sections, you can find more details about how to customize
each form field type.

== #short-or-long[Buttons][How to customize a button?] <button>

By default, the @form.button.body[button's body] is placed inside of a @rect[rectangle element].
You might want, however, to remove this rectangle and use a different shape or none at all.
Or, alternatively, you can customize the rectangle's @rect.fill[fill color],
or @rect.radius[round its corners].
Via show and set rules, you can decide how buttons look.

```example
>>> #form[
#form.button[Click me!]

#show form.button: it => {
  show form.widget: block(inset: 5pt, it.body)
  it
}

#form.button[Click me!]
>>> ]
```

In this example, we used a block with an @block.inset[`inset`] of `{5pt}`
to avoid cutting off parts of the text.
In Typst, all @form.widget[widgets] are @block.clip[clipped],
so they must be large enough to accommodate the text inside,
which can be fixed by adding an inset.

Instead of replacing the rectangle, you can also use @reference:styling:show-rules[show-set rules]
to customize the existing rectangle:

```example
>>> #set page(width: 100pt)
>>> #set align(center)
>>> #form[
#show form.button: it => {
  show form.widget: set rect(
    fill: aqua,
    stroke: 2pt + aqua.darken(30%),
    outset: -1pt,
    inset: (x: 7pt, y: 10pt),
    radius: 5pt,
  )
  it
}

#form.button[Click me!]
>>> ]
```

Through the set rule, we have changed the fill color, the stroke, the inset,
and the corner radius of the rectangle.
We have also changed the @rect.outset[outset] to avoid part of the stroke being
cut off due to @block.clip[clipping]: inside a widget,
it should be set to the negative half of the stroke length.

Additionally, you might want to add an interactive feel to the buttons by changing
the fill color when a user hovers their cursor over them.
As the buttons contain widgets with different states, you can write a show rule that
only targets the hovered state:

```typ
#show form.button: it => {
  show form.widget: // ... previous set rule
  show form.widget.where(state: "hovered"): set rect(fill: aqua.darken(10%))
  it
}
```

However, keep in mind that not all PDF readers support different appearances when hovering
or clicking (down). Please refer to the @form:compatibility[compatibility table].

The button element has a total of 3 states which can be customized via show rules
with the appropriate selector:

#docs-table(
  table.header[State][Selector],
  [Normal],
  `{form.widget.where(state: "normal")}`,
  [Hovered],
  `{form.widget.where(state: "hovered")}`,
  [Down],
  `{form.widget.where(state: "down")}`,
)

== #short-or-long[Checkboxes & Radio][How to customize a checkbox or radio button?] <checkbox-radio>

By default, a checkbox is shown as an empty square when unchecked, and as
a square with a `{sym.checkmark}` symbol inside when checked.
You might want, however, to change the checkmark symbol for a cross,
or replace the entire square with an emoji.
Similarly, radio buttons are shown as an empty circle when unselected,
and as a circle with a filled circle inside when selected.
Via show and set rules, you can decide how checkboxes and radio buttons look.

```example
>>> #form[
#form.checkbox(checked: true)

#show form.checkbox: it => {
  show sym.checkmark: sym.crossmark
  it
}

#form.checkbox(checked: true)
>>> ]
```

In this example, we use a show rule to replace the checkmark symbol with a crossmark symbol.
Since we only want to apply this replacement inside checkboxes and not for the entire document,
we use a nested show rule.
The outer show rule is for @form.checkbox, which has a function after the colon.
This function receives the checkbox element as its only argument, which we named `it`.
In the body of the function we apply the inner show rule, replacing the symbol for the one we
want, and then we return the element unchanged.

Instead of changing the checkmark symbol, you can also replace the entire checkbox
with content of your choice for each of its states.

```example
>>> #form[
#show form.checkbox: it => {
  show form.widget.where(checked: false): emoji.crossmark
  show form.widget.where(checked: true): emoji.checkmark.box
  it
}

#form.checkbox()
#form.checkbox(checked: true)
>>> ]
```

Here, we have again a nested show rule, which defines the content
for each state of the checkbox.
Each of the inner show rules has a @function.where[`where` selector], targeting
only the states where the checkbox is unchecked or checked, respectively.
In the first inner show rule, we tell Typst to show the crossmark emoji
when the checkbox is unchecked, while in the second we tell it to show
the checkmark box emoji when checked.

#todo[show-set?]

Both the checkbox and the radio button elements have a total of 6 states
which can be customized via show rules with the appropriate selector:

#docs-table(
  table.header[Checked][State][Selector],
  [No],
  [Normal],
  `{form.widget.where(checked: false, state: "normal")}`,
  [No],
  [Hovered],
  `{form.widget.where(checked: false, state: "hovered")}`,
  [No],
  [Down],
  `{form.widget.where(checked: false, state: "down")}`,
  [Yes],
  [Normal],
  `{form.widget.where(checked: true, state: "normal")}`,
  [Yes],
  [Hovered],
  `{form.widget.where(checked: true, state: "hovered")}`,
  [Yes],
  [Down],
  `{form.widget.where(checked: true, state: "down")}`,
)

== #short-or-long[Hover & Down States][How to customize the appearance when hovering or clicking a form field?] <hover-down>

Might be deleted if it fits into the two previous sections

== #short-or-long[Textboxes][How to customize a textbox?] <textbox>

Textboxes contain a single `{form.widget}`.
However, they also contain a `{form.cursor}`.

#todo[TODO]

== #short-or-long[Dropdowns & Listboxes][How to customize a dropdown or a listbox?] <choice>

Dropdowns and listboxes contain a single `{form.widget}`.

#todo[TODO]

= #short-or-long[Submission][How to allow users to submit a form?] <submission>

#todo[TODO]
