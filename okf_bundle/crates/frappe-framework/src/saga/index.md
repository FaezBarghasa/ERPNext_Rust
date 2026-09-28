# saga

## Classs

- [MockStep](MockStep.md)
- [SagaAction](SagaAction.md)
- [SagaCoordinator](SagaCoordinator.md)
- [SagaStatus](SagaStatus.md) — [derive(Clone, Debug, Serialize, Deserialize)]
- [SagaStep](SagaStep.md) — [derive(Clone, Debug, Serialize, Deserialize)]
- [SagaTransaction](SagaTransaction.md)

## Functions

- [compensate](compensate.md)
- [compensate](compensate_1.md)
- [default](default.md)
- [default](default_1.md)
- [execute](execute.md)
- [execute](execute_1.md)
- [execute_saga](execute_saga.md) — Executes a list of saga steps sequentially. On failure, triggers backward compensation.
- [execute_saga](execute_saga_1.md) — Executes a list of saga steps sequentially. On failure, triggers backward compensation.
- [new](new.md) — [must_use]
- [new](new_1.md) — [must_use]
- [test_saga_forward_and_compensating_rollback](test_saga_forward_and_compensating_rollback.md) — [test]
