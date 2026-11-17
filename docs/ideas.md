# Rupa ideas (our own direction)

Ideas that come from the author, not from herdr. Each has a status and the conditions under which it ships. Anything that changes scope goes into `TECH_SPEC.md` §7.2 before it is built (`AGENTS.md` §2).

## I1. Compact inbox window ("btop for agents")

**Idea (author):** a small always-open window that shows which agent needs which input, so the user can address it without hunting through panes.

**Why:** with many agents running, the cost is attention, not compute. Most users run agents on auto-approve, so what still stops an agent is a question, a choice, a plan to review, an error or "done", more than a permission prompt.

**Plan:**

| Version | Scope | Spec link |
|---|---|---|
| V1 | `rupa inbox`: read-only compact client (a mode of the normal client, attached as an observer). Per agent: status, the *kind* of block, the reason, the prompt text. One key jumps into the agent. | FR9, P4-S8 |
| V1.1 | Inline answer for questions and choices first; permission prompts later. | §7.2 "Permission inbox with inline approve" |
| Later | btop-style extras: per-agent CPU/memory, short activity history. | needs process probes |

**Conditions (do not skip):**
1. Show the full prompt text before any inline answer. No "approve all", no auto-approve.
2. Inline answering only ships after Gate G3 (status metrics measured), because a wrong status sends a keypress to the wrong prompt.
3. The key that answers is **per agent** (`1`, `y`, arrow plus Enter...). Each agent's mapping is an unverified assumption **[A]** until tested on the real CLI.
4. A floating OS widget is a GUI, which is a V1 non-goal. The terminal window covers the need.

## I2. Block reasons, not just "blocked"

**Idea (author's insight):** under auto-approve the interesting stops are not permissions, so the status needs a *kind*: `question`, `choice`, `plan-review`, `error`, `permission`, `done`.

**Open:** how reliably each kind can be told apart per agent is **[A]**, measured in P4-S10. Design the status output type with a `kind` field from the start (cheap now, expensive to retrofit).
