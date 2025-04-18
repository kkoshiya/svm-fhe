import { Connection, PublicKey } from '@solana/web3.js';
import { insertZero, encrypt, transfer, fhe8add } from './fhe';

const PROGRAM_ID = new PublicKey("5o9mxoRiUCtdd2JLvJGNoT5256mYBxEgG842b4M8pZDv");

async function startListener() {
    // Create connection
    const connection = new Connection('http://localhost:8899', {
        wsEndpoint: 'ws://localhost:8900',
        commitment: 'processed'
    });
    await insertZero();
    console.log('Starting listener...');

    // Subscribe to logs
    connection.onLogs(
        PROGRAM_ID,
        async (logInfo) => {
            // Check if there are any logs
            if (!logInfo.err && logInfo.logs.length > 0) {
                // Look for deposit instruction
                const depositLog = logInfo.logs.find(log => 
                    log.includes("User") && log.includes("deposited")
                );
                const depositInfoLog = logInfo.logs.find(log => 
                    log.includes("Deposit info:")
                );

                // Look for transfer logs
                const senderLog = logInfo.logs.find(log => 
                    log.includes("Sender's deposit value:")
                );
                const recipientLog = logInfo.logs.find(log => 
                    log.includes("Recipient's deposit value:")
                );
                const transferLog = logInfo.logs.find(log => 
                    log.includes("Transferring")
                );

                // FHE 8-bit add event detection
                const fheAddLhsLog = logInfo.logs.find(log => log.includes("FHE Add - LHS:"));
                const fheAddRhsLog = logInfo.logs.find(log => log.includes("FHE Add - RHS:"));
                const fheAddResultLog = logInfo.logs.find(log => log.includes("FHE addition result:"));

                if (depositLog && depositInfoLog) {
                    console.log('=== Deposit Detected ===');
                    console.log('Deposit Log:', depositLog);
                    const valueStr = depositLog.split("deposited ")[1].split(" ")[0];
                    const value = BigInt(valueStr);
                    const arrayStr = depositInfoLog.split("Deposit info:")[1].trim();
                    const ciphertext = JSON.parse(arrayStr);
                    await encrypt(value, ciphertext);
                    console.log('Deposit Amount:', value);
                    console.log('Deposit Info Array:', ciphertext);
                }

                if (senderLog && recipientLog && transferLog) {
                    console.log('=== Transfer Detected ===');
                    
                    // Extract sender ciphertext (handling debug format)
                    const senderArray = senderLog.split("value: ")[1].trim()
                        .replace(/[\[\]]/g, '');  // Remove brackets
                    const senderCiphertext = JSON.parse(`[${senderArray}]`);
                    
                    // Extract recipient ciphertext
                    const recipientArray = recipientLog.split("value: ")[1].trim()
                        .replace(/[\[\]]/g, '');
                    const recipientCiphertext = JSON.parse(`[${recipientArray}]`);
                    
                    // Extract transfer amount ciphertext
                    const transferArray = transferLog.split("Transferring ")[1].split(" from")[0].trim()
                        .replace(/[\[\]]/g, '');
                    const transferCiphertext = JSON.parse(`[${transferArray}]`);

                    await transfer(senderCiphertext, recipientCiphertext, transferCiphertext);
                    
                    console.log('Sender Ciphertext:', senderCiphertext);
                    console.log('Recipient Ciphertext:', recipientCiphertext);
                    console.log('Transfer Amount Ciphertext:', transferCiphertext);
                }

                if (fheAddLhsLog && fheAddRhsLog && fheAddResultLog) {
                    console.log('=== FHE 8-bit Add Detected ===');

                    const lhsArray = fheAddLhsLog.split("FHE Add - LHS:")[1].trim().replace(/\[|\]/g, '');
                    const rhsArray = fheAddRhsLog.split("FHE Add - RHS:")[1].trim().replace(/\[|\]/g, '');
                    const resultArray = fheAddResultLog.split("FHE addition result:")[1].trim().replace(/\[|\]/g, '');
                    
                    const lhs = JSON.parse(`[${lhsArray}]`);
                    console.log('LHS:', lhs);
                    const rhs = JSON.parse(`[${rhsArray}]`);
                    console.log('RHS:', rhs);
                    const result = JSON.parse(`[${resultArray}]`);
                    console.log('RHS:', result);

                    await fhe8add(lhs, rhs, result);
                    console.log('Encrypted LHS and RHS for FHE 8-bit add');
                }
            }
        },
        'confirmed'
    );

    console.log('Listening for program logs... (Press Ctrl+C to exit)');
}

startListener().catch(console.error);

