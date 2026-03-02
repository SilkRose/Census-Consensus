Result code supports comment lines. To make a comment, start the line with: **//**.

#### Result Code Formatting

The result code determines the text that gets inserted into the chapter when it is
published.  The result code consists of **conditions** and **text**.  When voting is finished, the conditions are checked top-to-bottom.  The first condition to succeed determines what is published.

A condition line is a # followed by a condition.  Here are examples with explanations:

- **# A**: Option A got the most votes.
- **# A > 50%**: Option A got more than 50% of the votes.
- **# B > 40% AND C > 30%**: Option B got over 30% and C got over 30%.
- **# A > 1/3**: Option A got over 1/3 of all votes.
- **# A > B**: Option A got more votes than B.

After each condition line is the result text.  This is the text substituted into the chapter.  For example:

> # A > 90%
> // This text will be used when over 90% of participants pick A.
> Oh, wow! Everyone seems to think A is the best!
> # A > B
> // If less than 90% of participants pick A but A has more votes than B, this text will be used.
> Most people like A, but some people like B.

The result text can refer to the voting results. This is done using replacement strings. Replacement strings consist of an option identifier and a replacement identifier.  The replacement identifier is one of the following

- **vp**: The vote percent.  For example, 52%.
- **vcc**: The count of votes, such as 1,234,567.
- **vcw**: The count of votes, using a round number and a word. For example, 10 million.
- **p-**: Prepended to an identifier to indicate that this replacement string refers to the result in that place.  See the examples.
- **name**: The text of an option.
- **question**: The text of the question.

These must be used with the following symbols that get replaced by you when writing:

- **id**: The id for a known option.
- **.d**: The number of decimal places to show for numbers/percentages.
- **x**: The position for an unknown result.

Here are some examples of how to use them. (Read these in order, since most examples build on the previous one.)

- **%A[vp]%**: A is the option id and vp is vote percent. If A received 23% of the votes, this would display 23%.
- **%A[vp.2]%**: .2 indicates the number of decimal places of precision. If A received 23.75% of the vote, this would display 23.75%.
- **%B[vcc]%**: B is the option id and vcc is the vote count. For example, this would be 1,234,567 if B received that many votes.
- **%C[vcw]%**: C is the option and vcw is the vote count in words, as in '10 million' or '1 million'.
- **%C[vcw.1]%**: .1 is the number of decimal places of precision, as in '10.1 million' or '1.3 million'.
- **%3[p-name]%**: 3 and p- together mean the third place result, and name is the text.  If the top vote getter was 'Rarity', the second place vote getter was 'Fluttershy', and the third place vote getter was 'Applejack', this would display 'Applejack'.
- **%2[p-vp]%**: 2 and p- together mean the second place result, and vp is vote percent.  If the second place vote getter had 23% of the vote, this would display 23%.
- **%2[p-vp.2]%**: .2 means 2 decimal places of precision, for example, 23.43%.
- **%4[p-vcc]%**: 4 and p- together mean the fourth place vote getter, and vcc is the vote count. For example, if the fourth place vote getter received 21,657 votes, this would display 21,657.
- **%2[p-vcw]%**: 2 and p- together mean the second place vote getter, and vcw is count in words. If the second place vote getter received 40,000,000 votes, this would display 40 million.
- **%2[p-vcw.1]%**: .1 is the number of decimal places of precision, as in 40.1 million.
- **%[question]%**: The text of the question.

Here is a complete example.  The question was 'Is Pinkie Pie cute?' and the options were 'A: yes', 'B: no', and 'C: absolutely'.

> // If 'absolutely' has over 1/2 of all votes:
> # C > 1/2
> 
> Twilight looked at Pinkie Pie. "This first question is about you."
> 
> "Oh," Pinkie Pie said.
> 
> // this would get replaced with 26 million, as an example.
> "%A[vcw]% ponies voted that you are absolutely cute!" Twilight said.
> 
> // This would get replaced with a percentage, such as 50.2%.
> "Wow, that's %A[vp.1]% of all of Equestria!"
> 
> // On the other hand, if A wins by a landslide, then we use the following.
> # A > 90%
> 
> // This uses the question replacement.
> Twilight said, "The question was *%[question]%*"
> 
> Pinkie smiled. "What were the options?"
> 
> "They were: %A[name]%, %B[name]%, %C[name]%."
> 
> "Which one won?"
> 
> "%A[name]% with over 90% of all votes!"
> 
> // Since both A and C have similar connotation, we can use the replacements to have them share a result.
> // If C has more than half the votes or A has more than 90% of the votes, we will use the replacement text above and not consider this possibility.
> // If either A has over 40% of the votes or C has over 40% of the votes, we use the following text.
> // For example, we will use it if A, B, and C get 41%, 46%, and 13%, respectively.
> # A > 40% OR C > 40%
> 
> Twilight looked at Pinkie Pie. "This first question is about you."
> 
> "Oh," Pinkie Pie said.
> 
> // The first replacement would be replaced by 'yes' or 'absolutely', and the second would be the percentage, such as 43%.
> "%1[p-name]% won with %1[p-vcw]% ponies voting that you are cute!" Twilight said.
> 
> // If all we care about is an option winning, we just list that option.
> // This condition will not be considered if C has more than half the votes, A has more than 90% of the votes, or either A or C has more than 40% of the votes.
> // If none of those happen, and if B is the winner, then we use the following text.
> // For example, if A, B, and C get 34%, 45%, and 21%, respectively, we will use the following text.
> # B
> 
> Twilight looked at Pinkie Pie. "This first question is about you."
> 
> "Oh," Pinkie Pie said.
> 
> // this would get replaced with 26 million, as an example.
> "%B[vcw]% ponies voted that you aren't cute!" Twilight said, shocked.
> 
> Pinkie frowned.
> 
> // We can also compare options directly.
> // We will not consider this possibility if C has more than half the votes; A has more than 90% of the votes; either A or C has more than 40% of the votes; or B is the winner.
> // If none of those happen, then we use the following replacement text if C has more votes than B and B has more votes than A.
> // For example, we use this if A, B, and C get 29%, 32%, and 39%, respectively.
> # C > B AND B > A
> 
> Twilight looked at Pinkie Pie. "This first question is about you."
> 
> "Oh," Pinkie Pie said.
> 
> // this would get replaced with 26 million, as an example.
> "%A[vcw]% ponies voted that you are absolutely cute!" Twilight said.
> 
> "Wow, what came in second?" Pinkie asked.
> 
> // This would be replaced with: no and 22.56%
> "%B[name]% with %B[vp.2]% of ponies voting for it."
> 
> // Every outcome needs a result text.
> // So far, we have text for the following possibilities: C has more than half the votes; A has more than 90% of the votes; either A or C has more than 40% of the votes; B is the winner; or C got more votes than B and B got more votes than A.
> // It is still possible that none of those happened.  For example, if A, B, and C got 36%, 26%, and 38%, respectively; or if they got 35%, 33%, and 32%.
> // One way to be careful and make sure every possibility is covered is to have a condition for every winner.
> # A
> 
> Twilight looked at Pinkie Pie. "This first question is about you."
> 
> "Oh," Pinkie Pie said.
> 
> "%A[vcw]% ponies think you're cute!" Twilight said.
> 
> // This last condition wraps up our possibilities
> # C
> 
> Twilight looked at Pinkie Pie. "This first question is about you."
> 
> "Oh," Pinkie Pie said.
> 
> "%A[vcw]% ponies think you're absolutely cute!" Twilight said.
