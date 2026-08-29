# WP16 Privacy and Linkability Campaign

The audit scanned 46 generated artifacts and approximately 5.91 million
field/delimiter occurrences. None of the configured subject DID, case number,
raw credential, or salt markers were found.

For 1,000 synthetic subjects, an unchanged salt/resource/warrant produced a
same-subject correlation rate of 1.0. Rotating the salt, resource, warrant, or
epoch reduced exact deterministic correlation to 0.0 in this experiment. A
low-entropy unsalted dictionary and a leaked-salt dictionary were modeled as
fully recoverable; an unknown random salt was not.

Crypto-shredding cases separately record whether mapping, salt, or backup
material remains. Deleting only one item is not claimed to guarantee erasure if
another mapping or backup survives.

This is a quantified exposure/linkability experiment, not a GDPR-compliance
certification or a claim of anonymity against auxiliary information.
