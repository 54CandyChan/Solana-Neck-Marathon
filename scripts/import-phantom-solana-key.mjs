import fs from "fs";
import os from "os";
import path from "path";
import readline from "readline";
import bs58 from "bs58";
import { Keypair } from "@solana/web3.js";

const outputPath =
  process.argv[2] || path.join(os.homedir(), ".config", "solana", "id.json");

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

const question = (prompt) =>
  new Promise((resolve) => {
    rl.question(prompt, (value) => resolve(value.trim()));
  });

try {
  console.log("This tool runs locally on your computer.");
  console.log("Paste only the Solana private key exported from Phantom.");
  console.log("Do not paste it into any website or chat.");
  console.log();

  const phantomKey = await question("Paste Phantom Solana private key: ");
  if (!phantomKey) {
    throw new Error("No private key provided.");
  }

  const decoded = bs58.decode(phantomKey);

  let keypair;
  if (decoded.length === 64) {
    keypair = Keypair.fromSecretKey(decoded);
  } else if (decoded.length === 32) {
    keypair = Keypair.fromSeed(decoded);
  } else {
    throw new Error(
      `Unexpected key length: ${decoded.length} bytes. Expected 32 or 64 bytes after base58 decoding.`
    );
  }

  const publicKey = keypair.publicKey.toBase58();
  console.log(`Detected wallet address: ${publicKey}`);

  const confirm = await question(
    "If this matches your Phantom Solana wallet address, type YES to continue: "
  );

  if (confirm !== "YES") {
    throw new Error("Address not confirmed. No file was written.");
  }

  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, JSON.stringify(Array.from(keypair.secretKey)));

  console.log(`Saved Solana CLI keypair file to: ${outputPath}`);
} catch (error) {
  console.error(`Error: ${error.message}`);
  process.exitCode = 1;
} finally {
  rl.close();
}
