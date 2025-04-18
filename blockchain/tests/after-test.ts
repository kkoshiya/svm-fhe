import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Blockchain } from "../target/types/blockchain";
import { PublicKey, SystemProgram } from '@solana/web3.js';

describe("Secondary Test Suite", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Blockchain as Program<Blockchain>;

  it("Retrieves stored [u8; 32] value and decrypts", async () => {
    console.log("Checking balance for pubkey:", provider.publicKey.toString());
    
    const [depositInfoPDA] = PublicKey.findProgramAddressSync(
      [provider.publicKey.toBuffer()],
      program.programId
    );
    console.log("Deposit Info PDA:", depositInfoPDA.toString());

    const accountInfo = await program.account.depositInfo.fetch(depositInfoPDA);
    
    // Convert to proper byte array and format
    const valueArray = Array.from(accountInfo.value);
    console.log("Stored value ([u8; 32]):", 
      valueArray.map(b => b.toString()).join(', ')
    );

    // Call decrypt endpoint using same format as working endpoints
    try {
        console.log('Sending decrypt request...');
        const response = await fetch('http://localhost:3000/decrypt', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ key: valueArray })
        });

        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(`HTTP error! status: ${response.status}, body: ${errorText}`);
        }

        const data = await response.json();
        console.log("Decrypted value:", data.result);
    } catch (error) {
        console.error('Detailed decrypt error:', error);
        throw error;
    }
    
    console.log("Owner of this deposit:", accountInfo.owner.toString());
  });

  it("Withdraws stored lamports to caller", async () => {
    // Get vault PDA
    const [vaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault")],
        program.programId
    );
    
    // Get program data address for authority check
    const [programDataAddress] = PublicKey.findProgramAddressSync(
        [program.programId.toBuffer()],
        new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111")
    );

    // Check initial balances
    const vaultBalance = await provider.connection.getBalance(vaultPDA);
    const recipientBalance = await provider.connection.getBalance(provider.publicKey);
    console.log(`Initial vault balance: ${vaultBalance} lamports`);
    console.log(`Initial recipient balance: ${recipientBalance} lamports`);

    try {
        const amount = new anchor.BN(200000); // 0.0002 SOL
        
        await program.methods
          .withdraw(amount)
          .accounts({
            vault: vaultPDA,
            authority: provider.publicKey,
            recipient: provider.publicKey,
            programData: programDataAddress,
            systemProgram: SystemProgram.programId,
          })
          .rpc();

        // Check final balances
        const newVaultBalance = await provider.connection.getBalance(vaultPDA);
        const newRecipientBalance = await provider.connection.getBalance(provider.publicKey);
        console.log(`Final vault balance: ${newVaultBalance} lamports`);
        console.log(`Final recipient balance: ${newRecipientBalance} lamports`);
        console.log(`Vault balance change: ${newVaultBalance - vaultBalance} lamports`);
        console.log(`Recipient balance change: ${newRecipientBalance - recipientBalance} lamports`);

    } catch (error) {
        console.error('Withdraw error:', error);
        throw error;
    }
  });
});


