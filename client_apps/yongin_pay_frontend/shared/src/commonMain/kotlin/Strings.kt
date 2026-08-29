// Root package

object Strings {
    var isKorean = true // Default to Korean

    val appName: String get() = if (isKorean) "용인 페이" else "Yongin Pay"
    val welcome: String get() = if (isKorean) "환영합니다," else "Welcome,"
    val send: String get() = if (isKorean) "송금" else "SEND"
    val request: String get() = if (isKorean) "요청" else "Request"
    val exchange: String get() = if (isKorean) "환전" else "Exchange"
    val recentTx: String get() = if (isKorean) "최근 결제 내역" else "RECENT TRANSACTIONS"
    
    val kioskMode: String get() = if (isKorean) "가맹점 키오스크 모드" else "Kiosk Mode"
    val hqMode: String get() = if (isKorean) "HQ 통합 관제 모드" else "HQ Dashboard Mode"
    val mintStablecoin: String get() = if (isKorean) "스테이블코인 발행 (현금 수납)" else "Mint Stablecoin (Cash In)"
    val burnStablecoin: String get() = if (isKorean) "스테이블코인 환전 (현금 지급)" else "Redeem Stablecoin (Cash Out)"
    val networkHealth: String get() = if (isKorean) "네트워크 상태" else "Network Health"
    val activeNodes: String get() = if (isKorean) "활성 노드" else "Active Nodes"
    val executeWarrant: String get() = if (isKorean) "전자영장 집행" else "Execute Warrant"
    val fdsAlerts: String get() = if (isKorean) "의심 거래 탐지 (FDS)" else "FDS Alerts"
    val tps: String get() = if (isKorean) "초당 트랜잭션 (TPS)" else "Transactions Per Sec (TPS)"
    
    val txHistory: String get() = if (isKorean) "거래 내역" else "Transaction History"
    val enterAmount: String get() = if (isKorean) "금액 입력" else "Enter Amount"
    val next: String get() = if (isKorean) "다음" else "Next"
    val scanQr: String get() = if (isKorean) "고객의 DID QR코드를 스캔해 주세요" else "Please scan customer's DID QR code"
    val mintComplete: String get() = if (isKorean) "발행 완료 (현금 수납 확인)" else "Mint Complete (Cash Received)"
    val receipt: String get() = if (isKorean) "영수증" else "Receipt"
    val backToHome: String get() = if (isKorean) "처음으로" else "Back to Home"
    
    fun toggleLanguage() {
        isKorean = !isKorean
    }
}
