-------------------------- MODULE ElectronicWarrant --------------------------
EXTENDS Naturals, FiniteSets, Sequences

CONSTANTS MaxAmount, Expiry, MaxTime

States == {"Draft", "Submitted", "CredentialVerified", "Authorized", "Active",
           "PartiallyExecuted", "FullyExecuted", "Suspended", "Revoked",
           "Expired", "Released", "Rejected", "Failed"}
TerminalStates == {"FullyExecuted", "Revoked", "Expired", "Released", "Rejected", "Failed"}
CommittedStates == {"Active", "PartiallyExecuted", "FullyExecuted", "Released", "Expired", "Revoked"}
AdapterPublishedStates == CommittedStates \cup {"Suspended"}
ProgressOutcomes == CommittedStates \cup {"Suspended", "Rejected", "Failed"}

VARIABLES warrant_state, credentials_verified, authorized, domain_valid,
          nonce_used, activation_count, reserved, executed, released, now,
          adapter_committed, audit_written, last_action

vars == <<warrant_state, credentials_verified, authorized, domain_valid,
          nonce_used, activation_count, reserved, executed, released, now,
          adapter_committed, audit_written, last_action>>

Init == /\ warrant_state = "Draft"
        /\ credentials_verified = FALSE
        /\ authorized = FALSE
        /\ domain_valid = TRUE
        /\ nonce_used = FALSE
        /\ activation_count = 0
        /\ reserved = 0 /\ executed = 0 /\ released = 0
        /\ now = 0 /\ adapter_committed = FALSE /\ audit_written = FALSE
        /\ last_action = "Init"

Submit == /\ warrant_state = "Draft"
          /\ warrant_state' = "Submitted" /\ last_action' = "Submit"
          /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                         activation_count, reserved, executed, released, now,
                         adapter_committed, audit_written>>

VerifyCredentials == /\ warrant_state = "Submitted"
                     /\ warrant_state' = "CredentialVerified"
                     /\ credentials_verified' = TRUE
                     /\ last_action' = "VerifyCredentials"
                     /\ UNCHANGED <<authorized, domain_valid, nonce_used, activation_count,
                                    reserved, executed, released, now, adapter_committed,
                                    audit_written>>

RejectUnauthorized == /\ warrant_state \in {"Submitted", "CredentialVerified"}
                      /\ warrant_state' = "Rejected" /\ audit_written' = TRUE
                      /\ last_action' = "RejectUnauthorized"
                      /\ UNCHANGED <<credentials_verified, authorized, domain_valid,
                                     nonce_used, activation_count, reserved, executed,
                                     released, now, adapter_committed>>

Authorize == /\ warrant_state = "CredentialVerified"
             /\ credentials_verified /\ domain_valid /\ ~nonce_used
             /\ warrant_state' = "Authorized" /\ authorized' = TRUE
             /\ nonce_used' = TRUE /\ last_action' = "Authorize"
             /\ UNCHANGED <<credentials_verified, domain_valid, activation_count,
                            reserved, executed, released, now, adapter_committed,
                            audit_written>>

Activate == /\ warrant_state = "Authorized" /\ authorized /\ domain_valid
            /\ now < Expiry /\ activation_count = 0
            /\ warrant_state' = "Active" /\ reserved' = MaxAmount
            /\ activation_count' = 1 /\ adapter_committed' = TRUE
            /\ audit_written' = TRUE /\ last_action' = "Activate"
            /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                           executed, released, now>>

Quarantine == /\ warrant_state = "Authorized"
              /\ warrant_state' = "Suspended" /\ audit_written' = TRUE
              /\ last_action' = "Quarantine"
              /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                             activation_count, reserved, executed, released, now,
                             adapter_committed>>

Abort == /\ warrant_state = "Authorized"
         /\ warrant_state' = "Failed" /\ audit_written' = TRUE
         /\ last_action' = "Abort"
         /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                        activation_count, reserved, executed, released, now,
                        adapter_committed>>

ReviewQuarantine == /\ warrant_state = "Suspended" /\ activation_count = 0
                    /\ warrant_state' = "Rejected" /\ audit_written' = TRUE
                    /\ last_action' = "ReviewQuarantine"
                    /\ UNCHANGED <<credentials_verified, authorized, domain_valid,
                                   nonce_used, activation_count, reserved, executed,
                                   released, now, adapter_committed>>

Execute(amount) == /\ warrant_state \in {"Active", "PartiallyExecuted"}
                   /\ now < Expiry /\ amount > 0
                   /\ executed + amount <= reserved
                   /\ executed' = executed + amount
                   /\ warrant_state' = IF executed' = reserved
                                         THEN "FullyExecuted"
                                         ELSE "PartiallyExecuted"
                   /\ audit_written' = TRUE /\ last_action' = "Execute"
                   /\ UNCHANGED <<credentials_verified, authorized, domain_valid,
                                  nonce_used, activation_count, reserved, released,
                                  now, adapter_committed>>

Release == /\ warrant_state \in {"Active", "PartiallyExecuted"}
           /\ released' = reserved - executed
           /\ warrant_state' = "Released" /\ audit_written' = TRUE
           /\ last_action' = "Release"
           /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                          activation_count, reserved, executed, now, adapter_committed>>

Suspend == /\ warrant_state \in {"Active", "PartiallyExecuted"}
           /\ warrant_state' = "Suspended" /\ audit_written' = TRUE
           /\ last_action' = "Suspend"
           /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                          activation_count, reserved, executed, released, now,
                          adapter_committed>>

Revoke == /\ warrant_state \in {"Authorized", "Active", "PartiallyExecuted", "Suspended"}
          /\ warrant_state' = "Revoked" /\ released' = reserved - executed
          /\ audit_written' = TRUE /\ last_action' = "Revoke"
          /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                         activation_count, reserved, executed, now, adapter_committed>>

Tick == /\ now < MaxTime
        /\ now' = now + 1 /\ last_action' = "Tick"
        /\ UNCHANGED <<warrant_state, credentials_verified, authorized, domain_valid,
                       nonce_used, activation_count, reserved, executed, released,
                       adapter_committed, audit_written>>

Expire == /\ now >= Expiry
          /\ warrant_state \in {"Active", "PartiallyExecuted"}
          /\ warrant_state' = "Expired" /\ released' = reserved - executed
          /\ audit_written' = TRUE /\ last_action' = "Expire"
          /\ UNCHANGED <<credentials_verified, authorized, domain_valid, nonce_used,
                         activation_count, reserved, executed, now, adapter_committed>>

TerminalStutter == /\ warrant_state \in TerminalStates
                   /\ UNCHANGED vars

AuthorizationOutcome == Activate \/ Quarantine \/ Abort \/ Revoke

Next == Submit \/ VerifyCredentials \/ RejectUnauthorized \/ Authorize
        \/ AuthorizationOutcome \/ ReviewQuarantine
        \/ (\E amount \in 1..MaxAmount: Execute(amount))
        \/ Release \/ Suspend \/ Tick \/ Expire \/ TerminalStutter

SafetySpec == Init /\ [][Next]_vars

LiveSpec == SafetySpec
            /\ WF_vars(Submit)
            /\ WF_vars(VerifyCredentials \/ RejectUnauthorized)
            /\ WF_vars(Authorize \/ RejectUnauthorized)
            /\ WF_vars(AuthorizationOutcome)
            /\ WF_vars(Tick)
            /\ WF_vars(Expire)
            /\ WF_vars(ReviewQuarantine)

TypeInvariant == /\ warrant_state \in States
                 /\ reserved \in 0..MaxAmount
                 /\ executed \in 0..MaxAmount
                 /\ released \in 0..MaxAmount
                 /\ now \in 0..MaxTime
                 /\ activation_count \in 0..1
UnauthorizedExecution == warrant_state \in CommittedStates => authorized
NoReplay == activation_count <= 1
AmountBound == executed <= MaxAmount
Conservation == executed + released <= reserved
NoPostExpiryExecution == last_action = "Execute" => now < Expiry
RevocationSafety == warrant_state = "Revoked" => released = reserved - executed
DomainBinding == warrant_state \in CommittedStates => domain_valid
Atomicity == /\ adapter_committed => warrant_state \in AdapterPublishedStates
             /\ (activation_count = 1 /\ warrant_state \in AdapterPublishedStates)
                 => adapter_committed
AuditCompleteness == warrant_state \in TerminalStates => audit_written
DomainNeutralCore == TRUE \* Enforced by the separate ARCH-005/010 static checks.

AuthorizedExecutionProgress == (warrant_state = "Authorized") ~> (warrant_state \in ProgressOutcomes)
ExpirationProgress == (warrant_state \in {"Active", "PartiallyExecuted"} /\ now >= Expiry)
                      ~> (warrant_state \in {"Expired", "Released", "FullyExecuted", "Revoked"})
QuarantineReview == (warrant_state = "Suspended") ~> (warrant_state \in {"Rejected", "Revoked"})

=============================================================================
