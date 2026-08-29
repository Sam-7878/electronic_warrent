# Model counterexamples resolved during WP11

Two model-definition counterexamples were found before the final publication runs.

1. `Atomicity` incorrectly required an Adapter commit for a warrant revoked before activation. The property was narrowed to distinguish authorization-only revocation from an activated reservation.
2. `ExpirationProgress` incorrectly required only `Expired`, although the executable policy permits a release, full execution, or revocation at the same boundary. The liveness consequent now accepts the documented terminal outcomes.

Neither counterexample exposed a Rust implementation defect. Both changed the formal property/model and are retained in `WP11_TLC_MODEL_CHECK.md`; final raw TLC stdout is under `formal/results/tlc/`.
