# fifo

## Classs

- [FifoBatchItem](FifoBatchItem.md) — An individual FIFO inventory batch layer.
- [InventoryError](InventoryError.md) — Inventory domain errors.
- [StockLedgerEntry](StockLedgerEntry.md) — Immutable Stock Ledger Entry (SLE) recording physical inventory movement.

## Functions

- [add_fifo_layer](add_fifo_layer.md) — Adds incoming receipt batch layer to FIFO queue.
- [consume_fifo](consume_fifo.md) — Consumes quantities from a FIFO batch queue sequentially, calculating exact COGS.
