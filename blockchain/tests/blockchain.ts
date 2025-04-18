import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Blockchain } from "../target/types/blockchain";
import { PublicKey } from '@solana/web3.js';

describe("blockchain", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Blockchain as Program<Blockchain>;
  const newUser = anchor.web3.Keypair.generate();
  const provider = anchor.getProvider();

  it("Can deposit SOL", async () => {
    const amount = new anchor.BN(1_000_000_000);
    
    const [vaultPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    );

    // Get PDA for deposit info
    const [depositInfoPDA] = PublicKey.findProgramAddressSync(
      [provider.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .deposit(amount)
      .accounts({
        depositInfo: depositInfoPDA,
        vault: vaultPDA,
        user: provider.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Deposit transaction signature", tx);
  });

  it("Can deposit SOL from another account", async () => {
    // Create new account with some SOL
    
    const signature = await provider.connection.requestAirdrop(
        newUser.publicKey,
        2_000_000_000  // 2 SOL (extra for fees)
    );
    await provider.connection.confirmTransaction(signature);

    // Get PDAs
    const [vaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault")],
        program.programId
    );

    const [depositInfoPDA] = PublicKey.findProgramAddressSync(
        [newUser.publicKey.toBuffer()],  // Use new user's pubkey
        program.programId
    );

    // Deposit 1 SOL
    const amount = new anchor.BN(1_000_000_000);
    const tx = await program.methods
        .deposit(amount)
        .accounts({
            depositInfo: depositInfoPDA,
            vault: vaultPDA,
            user: newUser.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([newUser])  // Add new user as signer
        .rpc();

    console.log("Second deposit transaction signature", tx);
});


  xit("Can transfer SOL", async () => {
    // Use helper to generate random value
    const value = generateRandomBytes32();
    const amount = new anchor.BN(700_000_000);
    console.log("raw value from helper", value);

    await deposit(Number(amount), value);
    //console.log('Deposited: into db', {value: value});
    await sleep(10000);
    // Derive PDA using the value as seeds
    const [depositInfoPDA] = PublicKey.findProgramAddressSync(
      [provider.publicKey.toBuffer()],
      program.programId
    );

    // Generate new recipient
    const recipient = newUser.publicKey;

    // Then transfer (just emits events)
    const tx = await program.methods
        .transfer(value, recipient)
        .accounts({
            depositInfo: depositInfoPDA,
            user: provider.publicKey,
            recipient: recipient,
        })
        .rpc();

    console.log("Transfer transaction signature", tx);
    console.log("Random value used:", value);
    console.log("Recipient:", recipient.toString());
  });

  it("Can fhe 8 add", async () => {
    // Use helper to generate random value
    const value1 = generateRandomBytes32();
    const number1 = 7;
    const value2 = generateRandomBytes32();
    const number2 = 2;

    await encrypt8(number1, value1);
    await encrypt8(number2, value2);
    //console.log('Deposited: into db', {value: value});
    await sleep(10000);
    // Derive PDA using the value as seeds
    const [resultInfoPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("fhe"), provider.publicKey.toBuffer()],
      program.programId
    );

    // Then transfer (just emits events)
    const tx = await program.methods
        .fhe8Add(value1, value2)
        .accounts({
            resultInfo: resultInfoPDA,
            user: provider.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

    console.log("Transfer transaction signature", tx);
    console.log("Random value 1 used:", value1);
    console.log("Random value 2 used:", value2);
  });
});



/////////////////// helper functions ///////////////////


function toBytes32(input: number[] | string | Buffer): number[] {
  if (Array.isArray(input)) {
      // If array, pad or truncate to 32 bytes
      return Array(32).fill(0).map((_, i) => input[i] || 0);
  } 
  if (typeof input === 'string') {
      // If hex string, remove 0x and convert
      const hex = input.startsWith('0x') ? input.slice(2) : input;
      const bytes = Buffer.from(hex.padStart(64, '0'), 'hex');
      return Array.from(bytes);
  }
  // If buffer, convert to array
  return Array.from(Buffer.from(input).slice(0, 32).padEnd(32, 0));
}

function generateRandomBytes32(): number[] {
  return toBytes32(Array(32).fill(0).map(() => Math.floor(Math.random() * 256)));
}

const deposit = async (lamports: number, key: string) => {
  console.log('Depositing: into db');
  
  const value = BigInt(lamports);
  
  // Convert key string to bytes array
  const encoder = new TextEncoder();
  const keyBytes = new Uint8Array(32);
  console.log('key bytes', keyBytes);
  const encodedKey = encoder.encode(key);
  
  // Copy encoded key into fixed-size array, padding with zeros if needed
  keyBytes.set(encodedKey.slice(0, 32));  // Take first 32 bytes or pad with zeros
  console.log('offset keybytes', keyBytes)

  const requestBody = {
      value: Number(value),
      key: key  // Convert to regular array for JSON
  };

  console.log('Sending to Rust server:', requestBody);

  const response = await fetch('http://localhost:3000/post', {
      method: 'POST',
      headers: {
          'Content-Type': 'application/json',
      },
      body: JSON.stringify(requestBody)
  });
  console.log('Rust Server Response:', await response.text());
}

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

const encrypt8 = async (value: number, key: string) => {
  const requestBody = {
    value: Number(value),
    key: key  
  };

  const response = await fetch('http://localhost:3000/encrypt8', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(requestBody),
  });

  console.log('Rust Server Response:', await response.text());
};

