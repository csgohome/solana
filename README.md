## ✅ 附加說明 / Additional Notes

---

### 1. 🎲 **消除模偏差，確保均勻分佈**
### **Eliminates Modulo Bias, Ensures Uniform Distribution**

> 使用「拒絕採樣法」（Rejection Sampling）過濾掉會導致偏倚的數值，確保最終輸出的隨機數在目標範圍內（如 1–100,000）完全均勻分佈。  
> Uses *rejection sampling* to filter out values that would cause bias, ensuring the final output is *perfectly uniformly distributed* within the target range (e.g., 1–100,000).

🔹 適用於彩票、抽獎、NFT 鑄造等需「公平性」的場景。  
🔹 Ideal for lotteries, raffles, NFT minting — anywhere *fairness* is critical.

---

### 2. 🔐 **高熵源組合，提升不可預測性**
### **High-Entropy Source Combination Enhances Unpredictability**

> 融合多個鏈上與客戶端熵源：訂單 ID、玩家公鑰、簽名者公鑰、區塊槽位、Unix 時間戳、最新區塊雜湊。  
> Combines multiple on-chain and client-provided entropy sources: Order ID, player pubkey, signer pubkey, slot, Unix timestamp, latest block hash.

🔹 即使部分熵源被礦工操縱（如時間戳），整體結果仍具高度隨機性。  
🔹 Even if some entropy sources are manipulated (e.g., timestamp), the *aggregate result remains highly random*.

---

### 3. 🧮 **使用 BLAKE3 雜湊，高效且抗碰撞**
### **Uses BLAKE3 Hash — Fast, Secure & Collision-Resistant**

> BLAKE3 是目前最快速且安全的密碼學雜湊函數之一，支援並行運算，輸出長度可變，抗長度擴展攻擊。  
> BLAKE3 is one of the fastest and most secure cryptographic hash functions — supports parallelism, variable output, and resists length-extension attacks.

🔹 比 SHA-256 更快，比 Keccak 更省 Gas（在 Solana 上雖無 Gas，但效能仍關鍵）。  
🔹 Faster than SHA-256, more efficient than Keccak — critical for on-chain performance.

---

### 4. 🔄 **雙階段雜湊 + 迭代計數器，避免重複與可預測性**
### **Two-Stage Hashing + Iteration Counter Prevents Repetition & Predictability**

> 第一階段雜湊所有熵源 → 第二階段加入迭代計數器再雜湊 → 生成多個隨機數時不會重複或可被逆推。  
> Stage 1: Hash all entropy → Stage 2: Re-hash with iteration counter → prevents repetition or reverse-engineering when generating multiple numbers.

🔹 即使同一訂單重複執行，只要迭代計數器不同，結果就不同。  
🔹 Even if same order is re-executed, different iteration counter = different result.

---

### 5. 📦 **動態指令格式，支援彈性輸入**
### **Dynamic Instruction Format Supports Flexible Input**

> 指令支援 UTF-8 字串（如訂單 ID、區塊雜湊），長度動態編碼（u32 little-endian），便於客戶端整合與擴展。  
> Instruction supports UTF-8 strings (e.g., order ID, block hash) with dynamic length encoding (u32 little-endian) — easy for client integration and future expansion.

🔹 可輕鬆支援未來新增熵源欄位（如 Chainlink VRF Seed、用戶自訂 Salt）。  
🔹 Easily extendable to support future entropy sources (e.g., Chainlink VRF seed, user-provided salt).

---

### 6. 🛡️ **嚴格帳戶驗證，防範未授權調用**
### **Strict Account Validation Prevents Unauthorized Invocation**

> 強制驗證玩家、簽名者、付費者皆為簽名者；Clock 與 System Program 帳戶需為官方合約地址。  
> Enforces that player, signer, and payer are *signers*; Clock and System Program accounts must be official addresses.

🔹 防止惡意合約偽造上下文或重放攻擊。  
🔹 Prevents malicious contracts from faking context or replay attacks.

---

### 7. 📈 **可擴展架構，支援批量生成**
### **Scalable Architecture Supports Batch Generation**

> 可一次生成最多 255 個隨機數（由 u8 count 控制），適合批量鑄造、多重抽獎等需求。  
> Can generate up to 255 random numbers in one call (controlled by u8 `count`) — ideal for batch minting or multi-winner draws.

🔹 減少交易次數與用戶手續費負擔（在支援費用的鏈上尤其重要）。  
🔹 Reduces transaction count and user fee burden — especially valuable on fee-based chains.

---

### 8. 📝 **清晰日誌輸出，便於除錯與審計**
### **Clear Logging Output for Debugging & Auditing**

> 使用 `msg!` 輸出訂單 ID 與生成的隨機數陣列，方便鏈上觀察與事後驗證。  
> Uses `msg!` to log Order ID and generated random numbers — easy to observe on-chain and verify post-execution.

🔹 開發者與審計人員可快速確認行為是否符合預期。  
🔹 Developers and auditors can quickly verify behavior matches specification.

## 🏁 總結 / Summary

| 優勢類別          | 繁體中文描述                     | English Description                     |
|------------------|----------------------------------|-----------------------------------------|
| 公平性           | 無偏倚、均勻分佈                 | Bias-free, uniform distribution         |
| 安全性           | 多熵源 + 密碼學雜湊              | Multi-entropy + cryptographic hash      |
| 效能             | BLAKE3 高速運算                  | BLAKE3 for high-speed computation       |
| 擴展性           | 支援批量、動態指令               | Supports batch, dynamic instructions    |
| 可審計性         | 清晰日誌 + 嚴格驗證              | Clear logs + strict validation          |
| 易用性           | 客戶端易整合、欄位彈性           | Easy client integration, flexible fields|

---

📌 **適用場景 / Ideal Use Cases**：  
NFT 鑄造抽籤 • 遊戲道具掉落 • DAO 抽選治理代表 • 社群活動抽獎 • 低風險娛樂型博弈

---

Copyright © 2025 CSGOHOME. All Rights Reserved.