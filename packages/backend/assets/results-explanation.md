Result writings support comment lines. To make a comment start the line with: **//**.

#### Result Writing Formatting

Result writings are the text that gets inserted into the chapter when published.
A result writing starts with a # followed by the condition for the vote answers.
Here are examples with explanations:

- **# A > 50%**: The option with id A won more than 50% of the votes.
- **# B > 40% AND C > 30%**: Option B got over 30% and C got over 30%.
- **# A > 1/3**: Option A won with over 1/3 of all votes.
- **# A > B**: Option A won with more votes than B.
- **A**: Option A won with the most votes.

As you can see above, result writings support both fractions and percentages.
Multiple conditions can be used, the first option that matches is the one that gets posted.
An example of result writings is as such:

> // if option A is over 30% this result will be put into the chapter.
> # A > 30%
> Oh, wow! Twilight, I can't believe you are so cute!
> // if A has more votes than B, but didn't pass the first writing condition, this will get posted.
> # A > B
> Oh, wow! Twilight and Pinkie are so cute!

Now, while writing your results, you might want
to directly quote the number or percentage of the votes or winning option.
This is supported with a set of replacement strings explained below:

Replacements use identifiers to work, the following is a list of all identifiers:

- **vp**: The vote percent.
- **vcc**: The count of votes. Ex: 1,234,567
- **vcw**: The count using the biggest word. Ex 10 million
- **p-**: Prepended to an identified for result placements where it is unknown.
- **name**: The text of an option.
- **question**: The text of the question.

These must be used with the following symbols that get replaced by you when writing:


- **id**: The id for a known option.
- **.d**: The number of decimal places to show for numbers/percentages.
- **x**: The position for an unknown result.

Here are some examples of how to use them:
(Each item explains what's new/changes from the previous one.)

- **%A[vp]%**: A is the option id, vp is vote percent. ex 23%
- **%A[vp.2]%**: .2 is the decimal places. ex 23.23%
- **%B[vcc]%**: B is the option id, vcc is the vote count. Ex: 1,234,567
- **%C[vcw]%**: C is the option, vcw is count in words. Ex 10 million
- **%C[vcw.1]%**: .1 is the decimal places. Ex 10.1 million
- **%3[p-name]%**: 3 is the placement, p- is placement prepend, name is the text.
- **%2[p-vp]%**: 2 is the placement, vp is vote percent. Ex 56%
- **%2[p-vp.2]%**: .2 is 2 decimal places. Ex 56.43%
- **%4[p-vcc]%**: 4 is the placement and vcc is the vote count. Ex 21,657,541
- **%2[p-vcw]%**: 2 is the placement and vcw is count in words. Ex 40 million
- **%2[p-vcw.1]%**: .1 is the decimal places. Ex 40.1 million
- **%[question]%**: The text of the question.

Here is a complete example:

The question is: Is Pinkie Pie cute?
The options are: A: yes, B: no, C: absolutely

// If C wins and has over 1/2 of all votes.
# C > 1/2

Twilight looked at Pinkie Pie. "This first question is about you."

"Oh," Pinkie Pie said.

// this would get replaced with 26 million, as an example.
"%A[vcw]% ponies voted that you are absolutely cute!" Twilight said.

// This would get replaced with 50.2%
"Wow, that's %A[vp.1]% of all of Equestria!"

// Since both A and C have similar connotation, we can use the replacements to have them share a result.
// If A or C wins and both have over 40% vote share.
# A > 40% OR C > 40%

Twilight looked at Pinkie Pie. "This first question is about you."

"Oh," Pinkie Pie said.

// this would get replaced with yes or absolutely, then with 32.2% for the second one.
"%1[p-name]% won with %1[p-vcw]% ponies voting that you are cute!" Twilight said.

// If all we care about is an option willing we can just list that option.
# B

Twilight looked at Pinkie Pie. "This first question is about you."

"Oh," Pinkie Pie said.

// this would get replaced with 26 million, as an example.
"%A[vcw]% ponies voted that you aren't cute!" Twilight said, shocked.

Pinkie frowned.

// We can also compare options directly.
// This would get used if absolutely won and no got more votes than yes.
# C > B AND B > A

Twilight looked at Pinkie Pie. "This first question is about you."

"Oh," Pinkie Pie said.

// this would get replaced with 26 million, as an example.
"%A[vcw]% ponies voted that you are absolutely cute!" Twilight said.

"Wow, what came in second?" Pinkie asked.

// This would be replaced with: no and 22.56%
"%B[name]% with %B[vp.2]% of ponies voting for it."

// If A wins by a landslide.
# A > 90%

// using the question replacement.
Twilight said, "The question was *%[question]%*"

Pinkie smiled. "What were the options?"

"They were: %A[name]%, %B[name]%, %C[name]%."

"Which one won?"

"%A[name]% with over 90% of all votes!"

