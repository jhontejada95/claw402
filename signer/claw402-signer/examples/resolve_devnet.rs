use claw402_policy::policy::USDC_DEVNET;
use claw402_signer::rpc::TrustedRpcClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = TrustedRpcClient::new("https://api.devnet.solana.com")?;
    let context = rpc.resolve_context(USDC_DEVNET)?;

    println!("network=solana-devnet");
    println!("mint={USDC_DEVNET}");
    println!("token_program={}", context.token_program);
    println!("mint_decimals={}", context.mint_decimals);
    println!("recent_blockhash={}", context.recent_blockhash);
    println!(
        "last_valid_block_height={}",
        context.last_valid_block_height
    );
    Ok(())
}
