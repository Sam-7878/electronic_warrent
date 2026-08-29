import logging
import time
import json

logging.basicConfig(level=logging.INFO, format='%(asctime)s [%(levelname)s] %(message)s')

# ==========================================
# 1. DID & VC/VP Objects
# ==========================================
class VerifiableCredential:
    def __init__(self, issuer_did: str, subject_did: str, claims: dict, signature: str):
        self.issuer_did = issuer_did
        self.subject_did = subject_did
        self.claims = claims
        self.signature = signature

class VerifiablePresentation:
    def __init__(self, vc: VerifiableCredential, subject_signature: str):
        self.vc = vc
        self.subject_signature = subject_signature

# ==========================================
# 2. Agent Classes
# ==========================================
class DIDIssuerAgent:
    """Offline KYC DB & VC Issuer (e.g., Shinhyup, Government)"""
    def __init__(self):
        self.did = "did:shinhyup:admin"
        self.offline_pii_db = {} # Maps DID -> PII (Not on Blockchain)

    def register_user(self, real_name: str, phone: str, address: str) -> str:
        user_did = f"did:shinhyup:user_{len(self.offline_pii_db)}"
        self.offline_pii_db[user_did] = {"name": real_name, "phone": phone, "address": address}
        logging.info(f"[DID Issuer] Registered offline PII for {real_name}. Generated {user_did}")
        return user_did

    def issue_vc(self, user_did: str, claims: dict) -> VerifiableCredential:
        logging.info(f"[DID Issuer] Issuing VC to {user_did} with claims: {claims}")
        return VerifiableCredential(self.did, user_did, claims, "issuer_crypto_sig_123")

    def process_warrant(self, target_did: str, warrant_sig: str) -> dict:
        """Called by Police/FSS with a legal warrant to extract PII"""
        logging.info(f"🏛️ [DID Issuer] Received Electronic Warrant for {target_did}. Verifying warrant...")
        if target_did in self.offline_pii_db:
            pii = self.offline_pii_db[target_did]
            logging.error(f"🏛️ [DID Issuer] Warrant verified! Releasing PII to Police: {pii}")
            return pii
        return {}

class AdminAgent:
    def issue_currency(self, target_did: str, amount: int, symbol: str, tag: str = None):
        logging.info(f"[Admin] Issued {amount} {symbol} to {target_did}. Tag: {tag}")
        return True

class UserAgent:
    def __init__(self, did: str):
        self.did = did
        self.vcs = []

    def receive_vc(self, vc: VerifiableCredential):
        self.vcs.append(vc)

    def pay(self, merchant: 'MerchantAgent', amount: int, token_symbol: str) -> bool:
        logging.info(f"[{self.did}] Attempting to pay {amount} {token_symbol} to {merchant.name}...")
        
        # Create VP from VC
        vp = None
        if self.vcs:
            vp = VerifiablePresentation(self.vcs[0], "user_biometric_sig_456")
        
        return api_gateway_mock_transfer(self.did, merchant.did, amount, merchant.category, vp)

class MerchantAgent:
    def __init__(self, name: str, category: str):
        self.name = name
        self.did = f"did:shinhyup:merchant_{name.replace(' ', '')}"
        self.category = category

    def request_settlement(self, amount: int):
        logging.info(f"[{self.name}] Requested settlement of {amount} to KRW via Credit Union.")
        return True

class AttackerAgent:
    def __init__(self, issuer: DIDIssuerAgent):
        # Hacker registers using fake/stolen identities to get DIDs
        self.wallets = []
        for i in range(5):
            did = issuer.register_user(f"StolenIdentity_{i}", "010-0000-0000", "Ghost City")
            self.wallets.append(did)
    
    def execute_cyclic_attack(self, target_merchant: MerchantAgent):
        logging.warning(f"😈 [Attacker] Initiating rapid cyclic transfer (깡) using {len(self.wallets)} DIDs...")
        for w in self.wallets:
            success = api_gateway_mock_transfer(w, target_merchant.did, 50000, target_merchant.category, None)

class FSSAgent:
    def query_pii(self, target_did: str, issuer: DIDIssuerAgent):
        logging.info(f"👮 [FSS/Police] Serving Electronic Warrant to DID Issuer for {target_did}...")
        pii = issuer.process_warrant(target_did, "police_warrant_sig_999")
        return pii

# ==========================================
# 3. System Mocks (Gateway & FDS & Core)
# ==========================================
frozen_dids = set()

def api_gateway_mock_transfer(sender_did: str, receiver_did: str, amount: int, merchant_category: str, vp: VerifiablePresentation) -> bool:
    # 1. Fast-Path FDS Check
    if sender_did in frozen_dids:
        logging.error(f"  -> ❌ [Gateway Fast-Path] REJECTED. {sender_did} is frozen.")
        return False

    # 1-5. Verify VP (Zero-Trust Privacy)
    if vp:
        logging.info(f"  -> 🔐 [Gateway] Verified VP. Subject claims: {vp.vc.claims}")
        if vp.vc.claims.get("eligible_for") == "BABY_SUBSIDY" and merchant_category != "BABY_PRODUCTS":
            logging.error(f"  -> ❌ [Rust Core] REJECTED. VP restricts usage to BABY_PRODUCTS.")
            return False
    elif merchant_category == "RESTRICTED_ADULT_ENTERTAINMENT":
         logging.error(f"  -> ❌ [Rust Core] REJECTED. Merchant category restricted.")
         return False

    # 3. Slow-Path FDS Stream (Async)
    if "StolenIdentity" in sender_did:
        logging.warning(f"  -> 🚨 [FDS Slow-Path] FRAUD DETECTED on {sender_did}! Executing Auto-Lock.")
        frozen_dids.add(sender_did)
        logging.info(f"  -> 🌐 [Compliance Level 1] Broadcasted {sender_did} metadata to Federation.")
    else:
        logging.info(f"  -> ✅ [Rust Core] SUCCESS. Transfer complete.")
        
    return True

# ==========================================
# 4. Execution of Scenarios
# ==========================================
def run_scenarios():
    issuer = DIDIssuerAgent()
    admin = AdminAgent()
    
    # Alice KYC Registration
    alice_did = issuer.register_user("Alice Kim", "010-1234-5678", "Yongin-si, Gyeonggi-do")
    alice = UserAgent(alice_did)

    print("\n" + "="*60)
    print("SCENARIO 1: Standard Local Currency Flow (DID Based)")
    print("="*60)
    # Alice gets a standard citizen VC
    citizen_vc = issuer.issue_vc(alice_did, {"is_citizen": True})
    alice.receive_vc(citizen_vc)
    
    merchant_restaurant = MerchantAgent("Bob's Diner", "RESTAURANT")
    admin.issue_currency(alice_did, 100000, "YONGIN_PAY")
    if alice.pay(merchant_restaurant, 30000, "YONGIN_PAY"):
        merchant_restaurant.request_settlement(30000)

    print("\n" + "="*60)
    print("SCENARIO 2: Purpose-Bound Subsidy Control (VP Checking)")
    print("="*60)
    # Alice gets a specific VC for Childcare Subsidy
    baby_vc = issuer.issue_vc(alice_did, {"eligible_for": "BABY_SUBSIDY"})
    alice.vcs = [baby_vc] # Replace active VC
    
    admin.issue_currency(alice_did, 500000, "CHILDCARE_SUBSIDY")
    merchant_club = MerchantAgent("Night Club", "RESTRICTED_ADULT_ENTERTAINMENT")
    
    # VP prevents this transaction at the Gateway/Core level
    alice.pay(merchant_club, 100000, "CHILDCARE_SUBSIDY")
    
    merchant_baby = MerchantAgent("Baby Care Shop", "BABY_PRODUCTS")
    alice.pay(merchant_baby, 50000, "CHILDCARE_SUBSIDY")

    print("\n" + "="*60)
    print("SCENARIO 3: Hybrid FDS & Electronic Warrant Execution")
    print("="*60)
    attacker = AttackerAgent(issuer)
    merchant_cashout = MerchantAgent("Shady Gold Shop", "JEWELRY")
    fss = FSSAgent()
    
    # Attacker executes cyclic transfer, gets frozen by nGNN FDS
    attacker.execute_cyclic_attack(merchant_cashout)
    
    # FSS (Police) steps in with a warrant for the frozen DID
    frozen_hacker_did = attacker.wallets[0]
    pii = fss.query_pii(frozen_hacker_did, issuer)
    logging.info(f"👮 [Police Action] Arresting suspect at address: {pii.get('address')}")

if __name__ == "__main__":
    run_scenarios()
