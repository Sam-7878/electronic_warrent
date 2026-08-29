# Electronic Warrant Fixtures

The jurisdiction example in `protocol/examples/electronic_warrant/` maps investigation, prosecutorial review, and judicial issuance to configurable roles. Institution names are deliberately absent from Rust core enums.

Functional valid/invalid fixture declarations are maintained in `evaluation/fixtures/electronic_warrant/scenarios.json` so the independent Python oracle and Rust test runner consume one stable scenario inventory.
