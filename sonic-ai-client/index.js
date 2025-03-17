const { Connection, PublicKey, Keypair, Transaction, TransactionInstruction, SystemProgram, sendAndConfirmTransaction, SYSVAR_RENT_PUBKEY } = require('@solana/web3.js');
const borsh = require('borsh');
const fs = require('fs');
const homedir = require('os').homedir();

// Load wallet from Solana config
const keypairPath = `${homedir}/.config/solana/id.json`;
const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
const wallet = Keypair.fromSecretKey(new Uint8Array(keypairData));

// Program ID from your deployment
const programId = new PublicKey('BCp9BaReGXHGPfcYFm7YdtKdBC2x9i3gvskgtiEKXJvk');
const connection = new Connection('https://api.testnet.sonic.game', 'confirmed');

class RegisterAgentArgs {
  constructor() {}
  
  serialize() {
    // RegisterAgent variant index is 0 and has no fields
    return Buffer.from([0]);
  }
}

class DepositCreditsArgs {
  constructor(amount) {
    this.amount = amount;
  }
  
  serialize() {
    const buffer = Buffer.alloc(9); // 1 byte for variant + 8 bytes for u64
    buffer[0] = 5; // DepositCredits variant index
    buffer.writeBigUInt64LE(BigInt(this.amount), 1);
    return buffer;
  }
}

async function registerAgent() {
  const agentAccount = Keypair.generate();
  
  const space = 32 + 8 + 4 + 4 + 1;
  console.log('Account space:', space);
  
  const lamports = await connection.getMinimumBalanceForRentExemption(space);
  console.log('Lamports needed:', lamports);
  
  // Get recent blockhash first
  const { blockhash } = await connection.getLatestBlockhash();
  
  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: wallet.publicKey,
    newAccountPubkey: agentAccount.publicKey,
    lamports,
    space,
    programId
  });

  // Debug the create account instruction
  console.log('\nCreate Account Instruction:');
  console.log('From:', wallet.publicKey.toString());
  console.log('New Account:', agentAccount.publicKey.toString());
  console.log('Program ID:', programId.toString());
  
  const instructionData = Buffer.from(new Uint8Array([0]));
  console.log('\nInstruction Data (hex):', instructionData.toString('hex'));
  console.log('Instruction Data (bytes):', [...instructionData]);
  
  const registerIx = new TransactionInstruction({
    keys: [
      { pubkey: agentAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: wallet.publicKey, isSigner: true, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ],
    programId,
    data: instructionData
  });

  // Debug the register instruction
  console.log('\nRegister Instruction Account Keys:');
  registerIx.keys.forEach((key, i) => {
    console.log(`Account ${i}:`, {
      pubkey: key.pubkey.toString(),
      isSigner: key.isSigner,
      isWritable: key.isWritable
    });
  });

  try {
    const tx = new Transaction();
    tx.recentBlockhash = blockhash;
    tx.feePayer = wallet.publicKey;
    tx.add(createAccountIx, registerIx);
    
    console.log('\nTransaction serialized size:', tx.serialize().length);
    
    const signature = await sendAndConfirmTransaction(
      connection, 
      tx, 
      [wallet, agentAccount],
      {preflightCommitment: 'confirmed'}
    );
    
    console.log('Transaction successful!');
    console.log('Signature:', signature);
    return agentAccount;
  } catch (err) {
    console.error('\nDetailed error information:');
    console.error('Error name:', err.name);
    console.error('Error message:', err.message);
    if (err.logs) {
      console.error('Transaction logs:', err.logs);
    }
    throw err;
  }
}

async function depositCredits(agentAccount, amount) {
  const { blockhash } = await connection.getLatestBlockhash();
  
  const depositIx = new TransactionInstruction({
    keys: [
      { pubkey: agentAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: wallet.publicKey, isSigner: true, isWritable: true }
    ],
    programId,
    data: new DepositCreditsArgs(amount).serialize()
  });

  const tx = new Transaction();
  tx.recentBlockhash = blockhash;
  tx.feePayer = wallet.publicKey;
  tx.add(depositIx);
    
  const signature = await sendAndConfirmTransaction(connection, tx, [wallet]);
  
  console.log(`Deposited ${amount} credits to agent`);
  console.log('Signature:', signature);
}

async function main() {
  try {
    console.log('Creating new AI agent...');
    const agent = await registerAgent();
    
    console.log('\nDepositing credits...');
    await depositCredits(agent, 1000);
    
    console.log('\nAgent is ready for inference!');
  } catch (err) {
    console.error('Error:', err);
  }
}

main();