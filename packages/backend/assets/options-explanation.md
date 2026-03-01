All option types support comment lines. To make a comment start the line with: **//**.

#### Scale Option Formatting

A scale question would be like this: 'On a scale from 1 to 10, how much do you love Pinkie Pie?'.

Scale options consist of two lines:
- **[1-10]**: The first number is the start of the scale, and the second the end of the scale.
- **Order: 5, 4, 2, 3, 1, 7, 6, 8, 10, 9**: An ordering of the options for priority to prevent ties.

An example of a scale option would be:

> // the scale options:
> [1-5]
> // the order in which to break ties:
> Order: 3, 2, 1, 5, 4

#### Multiple Choice/Multi-Select Option Formatting

These question types share the same option formatting.
The only difference is that Multiple Choice can only have one answer selected, 
while Multi-Select questions can have multiple answers checked.

These question types have two option line types:
- **A: [option text]**: The A is the option ID, a colon and a space, then the text of the option.
- **Order: A, C, B, D**: An ordering of the options for priority to prevent ties.

An example of these options would be:


> // The first option:
> A: Pinkie Pie
> // The second option:
> B: Twilight Sparkle
> // The third option:
> C: Fluttershy
> // the order in which to break ties:
> Order: A, C, B
