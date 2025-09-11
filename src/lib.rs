#![allow(unexpected_cfgs)]

// 引入 Solana 程式開發所需的核心模組
// Import core modules required for Solana program development
use solana_program::{
    account_info::{next_account_info, AccountInfo}, // 帳戶資訊處理工具 / Account info utilities
    clock::Clock,                                   // 取得鏈上時間與區塊槽位 / Access on-chain time and slot
    entrypoint,                                     // 程式入口巨集 / Program entrypoint macro
    entrypoint::ProgramResult,                      // 程式執行結果類型 / Program execution result type
    msg,                                            // 日誌輸出巨集 / Logging macro
    program_error::ProgramError,                    // 自訂錯誤類型 / Custom error types
    pubkey::Pubkey,                                 // 公鑰類型 / Public key type
    sysvar::{clock, Sysvar},                        // 系統變數存取 / System variable access
};

// 引入 BLAKE3 雜湊演算法庫，用於生成高品質熵源
// Import BLAKE3 hash library for high-quality entropy generation
use blake3;

// 宣告本程式的唯一識別符（Program ID）
// Declare the unique identifier (Program ID) for this program
solana_program::declare_id!("CSgoqaRA4ckXXDZRfS87hCSPsAuST7KSeyx8okT49cM");

// 定義最大隨機數範圍：1 到 100,000
// Define the maximum random number range: 1 to 100,000
const MAX_RANDOM: u64 = 100000;

// 計算安全上限，用於 rejection sampling 消除模偏差
// Calculate safe upper bound for rejection sampling to eliminate modulo bias
const MAX_SAFE: u64 = u64::MAX - (u64::MAX % MAX_RANDOM);

/*
 * 使用「拒絕採樣」(rejection sampling) 消除模運算偏差，
 * 確保生成的隨機數在 1 到 100,000 之間均勻分佈，無偏倚。
 *
 * Use [rejection sampling] to eliminate modulo bias
 * and ensure that the generated random numbers are
 * uniformly distributed between 1 and 100,000 without bias.
 */
fn rejection_sampling(raw: u64) -> Option<u32> {
    // 若原始值小於安全上限，則接受並映射到目標範圍
    // If raw value is below safe bound, accept and map to target range
    if raw < MAX_SAFE {
        Some((raw % MAX_RANDOM) as u32 + 1)
    } else {
        // 否則拒絕採樣，避免分佈偏倚
        // Otherwise reject sample to avoid distribution bias
        None
    }
}

// 定義指令資料結構：用於解析客戶端傳入的指令參數
// Define instruction data structure: used to parse client-provided instruction parameters
#[derive(Debug, PartialEq)]
pub struct GenerateRandomInstruction {
    pub oid: String,   // 訂單/請求唯一識別符 / Order/request unique identifier
    pub latest: String, // 最新區塊雜湊或其他熵源識別 / Latest block hash or other entropy source identifier
    pub count: u8,     // 請求生成的隨機數個數（最大 255） / Number of random numbers to generate (max 255)
}

// 為指令結構體實現資料解包方法
// Implement data unpacking method for instruction struct
impl GenerateRandomInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // 最小指令長度檢查：至少包含兩個 u32 長度 + 一個 u8 = 9 位元組
        // Minimum instruction length check: at least two u32 lengths + one u8 = 9 bytes
        if input.len() < 9 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut offset = 0;

        // 解析 oid 字串長度（小端序 u32）
        // Parse oid string length (little-endian u32)
        let oid_len = u32::from_le_bytes(input[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // 檢查剩餘資料是否足夠包含 oid 字串 + latest 長度 + count
        // Check if remaining data is sufficient for oid string + latest length + count
        if input.len() < offset + oid_len + 4 + 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 解析 oid 字串（UTF-8 編碼）
        // Parse oid string (UTF-8 encoded)
        let oid = String::from_utf8(input[offset..offset + oid_len].to_vec())
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        offset += oid_len;

        // 解析 latest 字串長度（小端序 u32）
        // Parse latest string length (little-endian u32)
        let latest_len = u32::from_le_bytes(input[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        // 檢查剩餘資料是否足夠包含 latest 字串 + count
        // Check if remaining data is sufficient for latest string + count
        if input.len() < offset + latest_len + 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 解析 latest 字串（UTF-8 編碼）
        // Parse latest string (UTF-8 encoded)
        let latest = String::from_utf8(input[offset..offset + latest_len].to_vec())
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        offset += latest_len;

        // 解析 count（u8）
        // Parse count (u8)
        let count = input[offset];

        // 返回解析成功的指令結構體
        // Return successfully parsed instruction struct
        Ok(GenerateRandomInstruction { oid, latest, count })
    }
}

// 定義帳戶結構體：用於驗證和引用傳入的帳戶
// Define account struct: used to validate and reference passed-in accounts
#[derive(Debug)]
pub struct RandomAccounts<'a, 'b> {
    pub player: &'a AccountInfo<'b>,      // 玩家帳戶（簽名者） / Player account (signer)
    pub signer: &'a AccountInfo<'b>,      // 額外簽名者帳戶 / Additional signer account
    pub payer: &'a AccountInfo<'b>,       // 付費帳戶（簽名者） / Payer account (signer)
    pub clock: &'a AccountInfo<'b>,       // Clock 系統變數帳戶 / Clock sysvar account
    pub system_program: &'a AccountInfo<'b>, // System Program 帳戶 / System Program account
}

// 為帳戶結構體實現解析與驗證方法
// Implement parsing and validation methods for account struct
impl<'a, 'b> RandomAccounts<'a, 'b> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        // 建立帳戶迭代器
        // Create account iterator
        let accounts_iter = &mut accounts.iter();

        // 按順序解析帳戶
        // Parse accounts in order
        let player = next_account_info(accounts_iter)?;
        let signer = next_account_info(accounts_iter)?;
        let payer = next_account_info(accounts_iter)?;
        let clock = next_account_info(accounts_iter)?;
        let system_program = next_account_info(accounts_iter)?;

        // 驗證 player 是否為簽名者
        // Validate player is a signer
        if !player.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // 驗證 signer 是否為簽名者
        // Validate signer is a signer
        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // 驗證 payer 是否為簽名者
        // Validate payer is a signer
        if !payer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // 驗證 clock 帳戶是否為合法的 Clock Sysvar
        // Validate clock account is the legitimate Clock Sysvar
        if !clock::check_id(clock.key) {
            return Err(ProgramError::InvalidArgument);
        }

        // 驗證 system_program 是否為官方 System Program
        // Validate system_program is the official System Program
        if system_program.key != &solana_program::system_program::ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        // 返回驗證通過的帳戶結構體
        // Return validated account struct
        Ok(RandomAccounts {
            player,
            signer,
            payer,
            clock,
            system_program,
        })
    }
}

// 設定程式入口點
// Set program entrypoint
entrypoint!(process_instruction);

// 主處理函式：接收程式 ID、帳戶列表、指令資料
// Main processing function: receives program ID, account list, instruction data
#[allow(unused_variables)]
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // 解析指令資料
    // Parse instruction data
    let instruction = GenerateRandomInstruction::unpack(instruction_data)?;

    // 解析並驗證帳戶
    // Parse and validate accounts
    let accounts = RandomAccounts::parse(accounts)?;

    // 從 Clock 帳戶中提取鏈上時間與槽位資訊
    // Extract on-chain time and slot info from Clock account
    let clock = Clock::from_account_info(accounts.clock)?;

    // 準備熵源資料：組合多個變數以增強隨機性
    // Prepare entropy source data: combine multiple variables to enhance randomness
    let order_bytes = instruction.oid.as_bytes();           // 訂單 ID 位元組 / Order ID bytes
    let count_bytes = instruction.count.to_le_bytes();      // 請求個數位元組 / Count bytes
    let slot_bytes = clock.slot.to_le_bytes();              // 目前槽位位元組 / Current slot bytes
    let timestamp_bytes = clock.unix_timestamp.to_le_bytes(); // Unix 時間戳位元組 / Unix timestamp bytes
    let player_bytes = accounts.player.key.to_bytes();      // 玩家公鑰位元組 / Player pubkey bytes
    let signer_bytes = accounts.signer.key.to_bytes();      // 簽名者公鑰位元組 / Signer pubkey bytes
    let latest_block_bytes = instruction.latest.as_bytes(); // 最新區塊識別位元組 / Latest block identifier bytes

    // 預先分配緩衝區容量，避免多次記憶體配置
    // Pre-allocate buffer capacity to avoid multiple memory allocations
    let mut data = Vec::with_capacity(
        order_bytes.len()
            + count_bytes.len()
            + slot_bytes.len()
            + timestamp_bytes.len()
            + player_bytes.len()
            + signer_bytes.len()
            + latest_block_bytes.len(),
    );

    // 將所有熵源資料拼接進緩衝區
    // Concatenate all entropy source data into buffer
    data.extend_from_slice(order_bytes);
    data.extend_from_slice(&count_bytes);
    data.extend_from_slice(&slot_bytes);
    data.extend_from_slice(&timestamp_bytes);
    data.extend_from_slice(&player_bytes);
    data.extend_from_slice(&signer_bytes);
    data.extend_from_slice(&latest_block_bytes);

    // 初始化 BLAKE3 雜湊器，進行第一輪雜湊
    // Initialize BLAKE3 hasher, perform first round of hashing
    let mut hasher = blake3::Hasher::new();
    hasher.update(&data);
    let intermediate = hasher.finalize(); // 取得中間雜湊值 / Get intermediate hash value

    // 初始化結果陣列，預先分配容量
    // Initialize result array, pre-allocate capacity
    let mut arr: Vec<u32> = Vec::with_capacity(instruction.count as usize);
    let mut iteration_count: i32 = 0; // 迭代計數器，用於生成多個隨機數 / Iteration counter for generating multiple random numbers

    // 迴圈生成指定數量的隨機數
    // Loop to generate specified number of random numbers
    while arr.len() < instruction.count as usize {
        iteration_count += 1;

        // 重置雜湊器，進行第二輪雜湊（加入迭代計數器避免重複）
        // Reset hasher, perform second round of hashing (add iteration counter to avoid repetition)
        hasher.reset();
        hasher.update(intermediate.as_bytes());
        hasher.update(&iteration_count.to_le_bytes());
        let batch_hash = hasher.finalize();

        // 取雜湊值前 8 位元組作為 u64 原始隨機數
        // Take first 8 bytes of hash as u64 raw random number
        let bytes = batch_hash.as_bytes()[..8].try_into().unwrap();
        let raw = u64::from_le_bytes(bytes);

        // 使用拒絕採樣法生成無偏隨機數
        // Use rejection sampling to generate unbiased random number
        match rejection_sampling(raw) {
            Some(random_value) => {
                arr.push(random_value); // 採樣成功，加入結果陣列 / Sampling successful, add to result array
            }
            None => {
                break; // 採樣失敗，跳出迴圈（理論上機率極低） / Sampling failed, break loop (theoretically very low probability)
            }
        }
    }

    // 輸出日誌：列印訂單 ID 與生成的隨機數列表
    // Output log: print order ID and generated random number list
    msg!("ID={} RANDOMS={:?}", instruction.oid, arr);

    // 返回成功
    // Return success
    Ok(())
}