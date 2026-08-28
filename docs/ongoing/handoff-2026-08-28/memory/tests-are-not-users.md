---
name: tests-are-not-users
description: "A corpus count — any count, any population — is never evidence that a language need does or does not exist; only a program someone wants to write and cannot is"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-19T07:12:39.530Z
---

2026-08-18/19, Whitefoot, owner shouting twice in one session, the second time
because I had not learned it properly the first.

**First cut (wrong, too weak).** I justified the unlabelled `fn main()` entry
form by saying that removing it would force "451 pure-computation programs" to
change. The owner: those are TESTS, not programs — "测试是给编译器服务的，他们
不是程序". I recorded the lesson as "cite real programs, not the test corpus".

**Second cut (the actual rule).** One message later I approvingly quoted a
subagent arguing against a construct because it "would have three instances in
a 637-file corpus, all test cases". The owner, furious: "任何测试里的fact都不
可以拿来做论据…不管你是找到三个还是三百个，都无关紧要。这些东西不能代表任何
事情，不能代表现实当中这种需求不存在。"

So the rule is not "count the right population". It is: **a corpus count is not
evidence about language need at all.**

**Why:** what the tree contains is what the CURRENT language allowed someone to
write, for our own purposes, mostly to exercise the compiler. A capability that
does not exist yet has zero uses BY CONSTRUCTION — using that zero to argue
nobody needs it is circular. And the programs of the world, which the language
is for, are not in the tree at any count.

**What a corpus number CAN establish:** facts about the corpus itself — "this
construct is currently spelled these N ways", "this rule has no rejecting case",
"these bytes would stop parsing". Those are observations about the artifact.
They never become "therefore the language should / should not have X".

**What evidence for a language need looks like:** a program someone wants to
write and cannot, or can write only badly — demonstrated, not counted. One
worked example of a shape the language mangles outweighs any tally. See also
[[migration-cost-is-not-a-design-criterion]]: same failure family — an
accumulated artifact being allowed to vote on the language's future.
