All option types support comment lines. To make a comment, start the line with **//**.

#### Scale Option Formatting

A scale question asks participants to choose a number. For example, 'On a scale from 1 to 10, how much do you love Pinkie Pie?'. Or, 'How many hayburgers should Twilight eat?'

Scale options consist of two lines:

- **The numbers in the scale**.  This line is an opening square bracket [, then the number at the start of the scale, then a dash -, then the number at the end of the scale, then a closing square bracket ]. For example, [1-3] would allow participants to pick a number from the list one, two, three.
- **Tiebreakers**.  This line is the word Order: followed by a comma-separated list of numbers.  If there is a tie for the option with the greatest number of votes, then the winner will be whichever option comes first in this list. For example, 3, 1, 2 means, 'If 3 is tied for first place, then 3 is the winner. If 1 is tied for first place with any number except 3, then 1 is the winner. If 2 is tied for first place, then 2 is never the winner.'

Here is an example of a scale option:

> // Participants pick a number between 1 and 10
> [1-10]
> // Ties are broken in favor of 5, then in favor of 4, and so on
> Order: 5, 4, 2, 3, 1, 7, 6, 8, 10, 9

#### Multiple Choice and Multiple Selection Option Formatting

A multiple choice question is one where participants are given a list of choices and must pick a single choice. An example multiple choice question is, 'Who is your favorite Wonderbolt?' Possible answers are 'Rainbow Dash', 'Spitfire', 'Soarin', and so on, and participants may pick only one Wonderbolt as their favorite.

A multiple selection question is one where participants are given a list of choices and may select any number of answers they like. An example multiple selection question is 'Which CMCs deserve ice cream?' Possible answers might be 'Apple Bloom', 'Sweetie Belle', 'Scootaloo', 'Babs Seed', and 'Gabby'.  One participant might select 'Apple Bloom' and 'Babs Seed', a second might select all five, and a third might select none.

Multiple choice and multiple selection questions are entered the same way. They have two types of lines.

- **Option lines**. These lines consist of a single-letter option name, a colon and a space, and then the option text. There will normally be more than one option line. For example, a question which asks you to choose among the four princesses would have the lines: A: Princess Celestia, B: Princess Luna, C: Princess Cadance, and D: Princess Twilight Sparkle.
- **Tiebreakers**. This line is the word Order: followed by a comma-separated list of options. If there is a tie for the option with the greatest number of votes, then the winner will be whichever option comes first in this list. For example, C, B, D, A means, 'If C is tied for first place, then C is the winner. If B is tied for first place with any option except B, then B is the winner. If D is tied for first place with any options except C and B, then D is the winner. If A is tied for first place, then A is never the winner.'

Here is an example of a multiple choice or multiple select question:

> // The first option:
> A: Pinkie Pie
> // The second option:
> B: Twilight Sparkle
> // The third option:
> C: Fluttershy
> // Break ties in favor of Pinkie Pie, then in favor of Fluttershy
> Order: A, C, B
