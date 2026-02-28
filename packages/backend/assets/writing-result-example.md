// The question is: Is Pinkie Pie cute?
// The options are: A: yes, B: no, C: absolutely

// If C wins and has over 1/2 of all votes.
# C > 1/2

Twilight looked at Pinkie Pie. "This first question is about you."

"Oh," Pinkie Pie said.

// this would get replaced with 26 million, as an example.
"%A[vcw]% ponies voted that you are abosultely cute!" Twilight said.

// This would get replaced with 50.2%
"Wow, that's %A[vp.1]% of all of Equestria!"

// Since both A and C have similar conotation, we can use the replacements to have them share a result.
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

