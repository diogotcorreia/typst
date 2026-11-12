#import "../../components/index.typ": docs-chapter, short-or-long

#show: docs-chapter.with(
  title: "Form Guide",
  route: "/guides/forms",
  description: "Not sure how to give your checkboxes their own style? Or how to allow your users to submit your form? This guide explains all you need to know about interactive forms in Typst.",
)

Interactive forms are a great way to create PDFs that can be filled out by your users.

/*

Tables are a great way to present data to your readers in an easily readable, compact, and organized manner. They are not only used for numerical values, but also survey responses, task planning, schedules, and more. Because of this wide set of possible applications, there is no single best way to lay out a table. Instead, think about the data you want to highlight, your document's overarching design, and ultimately how your table can best serve your readers.

Typst can help you with your tables by automating styling, importing data from other applications, and more! This guide takes you through a few of the most common questions you may have when adding a table to your document with Typst. Feel free to skip to the section most relevant to you – we designed this guide to be read out of order.
*/

If you want to look up a detail of how forms or fields work, you should also @form[check out their reference page].

= #short-or-long[Basic Forms][How to create a basic form?] <basic-forms>
In order to create a form in Typst, use the @form[`form` function].
However, a form is of no use without fields the user can interact with.

For this example, let's say we want to create a room booking form,
where users can fill out their name and when they want to use the room.
We can use a textbox to ask for the requester's name,
and checkboxes to list the possible booking times:

```example
= Room Booking

#form[
  #form.label(<name>)[What is your name?]
  #form.textbox() <name>

  Which periods would you like to book the room for?
  #grid(
    columns: 2,
    inset: 0.2em,
    align: horizon,
    [#form.checkbox() <morning>],
    form.label(<morning>)[Morning],
    [#form.checkbox() <afternoon>],
    form.label(<afternoon>)[Afternoon],
    [#form.checkbox() <evening>],
    form.label(<evening>)[Evening],
  )
]
```

This example shows how to create a form with multiple fields, labels, and surrounding text.
The `form` function does not require any arguments other than the contents of the form,
which is passed as a @content[content block] using square brackets.
In the form's content, there are a total of 4 form fields,
one @form.textbox[textbox] and three @form.checkbox[checkboxes],
along with a @form.label for each field.

The textbox is preceded by its @form.label, and both elements are linked via the `{<name>}` @label.
The @form.label function takes the @label of the field it describes as its first positional argument,
and the description of the field as its second position argument,
passed in the example as a content block (`{[What is your name?]}`).

Not all text needs to be enclosed in @form.label.
In the example, the second question is placed in the body of the form directly
as it does not describe any single form field, but it is just filler text encompassing
the subsequent checkboxes.

To create an aligned list of checkboxes and labels, we use a @grid[grid].
Each row contains two columns: the leftmost has a checkbox, and the rightmost has the checkbox's @form.label.
Just like with the textbox, the two are linked via a @label (`{<morning>}`, `{<afternoon>}`, and `{<evening>}`).

Congratulations, you have created your first form!
Now you can explore other field types such as @form.radio-group and @form.choice,
or proceed to @guides:forms:styling[giving a unique style to your form fields],
@guides:forms:submission[allowing users to submit your form], and more!

= #short-or-long[Inlining][How to show fields inline?] <inlining>

All form fields are block-level elements and cannot occur inside of a @par[paragraph].
To change this behavior, we can use the @box[`box` function] to integrate form fields
into a paragraph:

```example
#form[
  #box(form.checkbox())
  This text is next to the checkbox
]
```

In practice, the `box` turns the checkbox into an inline element, and places it next
to the subsequent text.

However, having to wrap every form field in a @box can get burdensome when you have a dozen or more
form fields.
To ease this, we can use a show rule for the checkbox:

```example
#show form.checkbox: box

#form[
  #form.checkbox()
  This text is next to the checkbox
]
```

With this show rule, all checkboxes in the document will be placed inline!

// TODO: baseline? perhaps we should fix it by default

= #short-or-long[Styling][How to change the appearance of form fields?] <styling>

While Typst provides a default appearance for each field type,
you are free to customize them as you see fit through @reference:styling:show-rules[show rules]
as explained in this section.

// TODO: I made this up, not sure if pseudo-element is a good name, but I want to highlight that they are slightly special.
// Name comes from HTML's pseudo selectors.
// This whole section probably needs further rewriting to properly convey its message.
// A diagram or something like that may help, but I'm hesitant in showing typst code that doesn't really compile
Given their interactive nature, all fields exclusively contain one or more
@form.widget[`form.widget` pseudo-elements].
We call them pseudo-elements because, unlike regular elements, they are not
always visible on the document and instead represent a single state the field can be in.
For instance, a @form.checkbox's default show rule contains a total of 6 widgets:
the checkbox can either be unchecked or checked, and at the same time it can be unfocused, hovered, or being clicked on,
resulting in 6 different combinations.

When opening a PDF with interactive form fields, only one of these widgets is shown per field at a given time.
The widget that is currently visible can change when the field is interacted with.
For example, when an unchecked checkbox is clicked, it becomes checked and its `{form.widget(checked: false)}` is hidden
and its `{form.widget(checked: true)}` is shown.

The existence of these pseudo-elements allows the various states of each field to be
styled via show rules, even if those states are not active when the document is created.

== #short-or-long[Buttons][How to customize a button?] <button>

Buttons contain the following widgets:

- `{form.widget(state: "normal")}`
- `{form.widget(state: "hovered")}`
- `{form.widget(state: "down")}`

TODO

== #short-or-long[Checkboxes & Radio][How to customize a checkbox or radio button?] <checkbox-radio>

Checkboxes and radio buttons contain the following widgets:

- `{form.widget(checked: false, state: "normal")}`
- `{form.widget(checked: false, state: "hovered")}`
- `{form.widget(checked: false, state: "down")}`
- `{form.widget(checked: true, state: "normal")}`
- `{form.widget(checked: true, state: "hovered")}`
- `{form.widget(checked: true, state: "down")}`

TODO

== #short-or-long[Hover & Down States][How to customize the appearance when hovering or clicking a form field?] <hover-down>

Might be deleted if it fits into the two previous sections

== #short-or-long[Textboxes][How to customize a textbox?] <textbox>

Textboxes contain a single `{form.widget}`.
However, they also contain a `{form.cursor}`.

TODO

== #short-or-long[Dropdowns & Listboxes][How to customize a dropdown or a listbox?] <choice>

Dropdowns and listboxes contain a single `{form.widget}`.

TODO

= #short-or-long[Submission][How to allow users to submit a form?] <submission>

TODO
