# lifecycle

## Classs

- [Document](Document.md) — In-memory representation of a Frappe document instance.
- [DocumentController](DocumentController.md) — Unified Document Controller enforcing lifecycle state transitions and hooks.
- [DocumentError](DocumentError.md) — Document operation and lifecycle transition errors.

## Functions

- [cancel](cancel.md) — Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks.
- [cancel](cancel_1.md) — Cancels a submitted document, mutating docstatus to 2 and triggering reversal hooks.
- [default](default.md)
- [default](default_1.md)
- [insert](insert.md) — Handles document insertion: runs naming series, validation hooks, and commits to draft state.
- [insert](insert_1.md) — Handles document insertion: runs naming series, validation hooks, and commits to draft state.
- [is_cancelled](is_cancelled.md) — Is this document cancelled?
- [is_cancelled](is_cancelled_1.md) — Is this document cancelled?
- [is_draft](is_draft.md) — Is this document a draft?
- [is_draft](is_draft_1.md) — Is this document a draft?
- [is_submitted](is_submitted.md) — Is this document submitted?
- [is_submitted](is_submitted_1.md) — Is this document submitted?
- [new](new.md) — Creates a new draft document instance.
- [new](new_1.md) — Creates a new draft document instance.
- [new](new_2.md) — Creates a new DocumentController.
- [new](new_3.md) — Creates a new DocumentController.
- [submit](submit.md) — Submits a draft document, mutating docstatus to 1 and triggering submission hooks.
- [submit](submit_1.md) — Submits a draft document, mutating docstatus to 1 and triggering submission hooks.
- [update](update.md) — Mutates an existing document while rejecting edits to submitted documents.
- [update](update_1.md) — Mutates an existing document while rejecting edits to submitted documents.
