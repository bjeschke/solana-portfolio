use std::collections::HashMap;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::env;
use serde_json::Value;
use solana_client::rpc_request::{RpcRequest, TokenAccountsFilter};
use reqwest::blocking::get;
use spl_token::ID as TOKEN_PROGRAM_ID;
use solana_account_decoder::UiAccountData;


fn load_token_map() -> Result<HashMap<String, (String, String)>> {
    let url = "https://raw.githubusercontent.com/solana-labs/token-list/main/src/tokens/solana.tokenlist.json";
    let response: Value = get(url)?.json()?;

    let mut map = HashMap::new();

    if let Some(tokens) = response["tokens"].as_array() {
        for token in tokens {
            let mint = token["address"].as_str().unwrap_or("").to_string();
            let name = token["name"].as_str().unwrap_or("Unknown").to_string();
            let symbol = token["symbol"].as_str().unwrap_or("").to_string();
            map.insert(mint, (name, symbol));
        }
    }

    Ok(map)
}


fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <WALLET_ADDRESS>", args[0]);
        std::process::exit(1);
    }

    let wallet_address = &args[1];
    let pubkey = wallet_address.parse::<Pubkey>()?;

    // Mainnet RPC Endpoint (du kannst auch testnet oder devnet nehmen)
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    println!("🔍 Scanning tokens for address: {}\n", wallet_address);

    // SOL Balance
    let lamports = client.get_balance(&pubkey)?;
    let sol = lamports as f64 / 1_000_000_000.0;
    println!("💰 SOL: {:.4} SOL", sol);

    // SPL-Token-Konten als parsed JSON abfragen
    let params = serde_json::json!([
        wallet_address,
        {
            "programId": spl_token::id().to_string()
        },
        {
            "encoding": "jsonParsed"
        }
    ]);

    // RPC-Call: getTokenAccountsByOwner mit JSON-Ausgabe
    let response = client.send::<Value>(RpcRequest::GetTokenAccountsByOwner, params)?;
    let token_map = load_token_map()?;

    // Parsen und Anzeigen der Token
    let empty_vec = Vec::new(); // Sicherer Fallback
    let accounts = response["value"].as_array().unwrap_or(&empty_vec);

    if accounts.is_empty() {
        println!("⚠️  No SPL tokens found.");
    } else {
        println!("\n📦 SPL Tokens:");
        for account in accounts {
            let parsed = &account["account"]["data"]["parsed"];
            let info = &parsed["info"];
            let mint = info["mint"].as_str().unwrap_or("???");
            let amount_str = info["tokenAmount"]["uiAmountString"].as_str().unwrap_or("0");
            let ui_amount = info["tokenAmount"]["uiAmount"]
                .as_f64()
                .unwrap_or(0.0);

            // Nur anzeigen, wenn > 0
            if ui_amount > 0.0 {
                let (name, symbol) = token_map
                    .get(mint)
                    .cloned()
                    .unwrap_or(("Unknown".to_string(), "".to_string()));

                println!("• Token: {} ({})", name, mint);
                println!("• Mint: {}\n  Amount: {}", mint, amount_str);
            }
        }
    }

    Ok(())
}
