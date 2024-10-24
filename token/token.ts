import * as web3 from "@solana/web3.js"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { fromWeb3JsKeypair , fromWeb3JsPublicKey } from "@metaplex-foundation/umi-web3js-adapters";
import { createSignerFromKeypair, signerIdentity, Umi } from "@metaplex-foundation/umi";
import { CreateMetadataAccountV3InstructionAccounts, CreateMetadataAccountV3InstructionDataArgs , createMetadataAccountV3 } from "@metaplex-foundation/mpl-token-metadata";
import fs from 'fs';

export function loadWalletKey(keypairFile: string): web3.Keypair {
    const loaded = web3.Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(fs.readFileSync(keypairFile).toString())),
    );
    return loaded;
}


async function main() {
    console.log("Let's name some tokens");

    const myKeypair = loadWalletKey("owQjAAczpK3Mk9euh8Ax2vbiPt9BDh4wWiHMz9TV22c.json")
    console.log(myKeypair.publicKey.toBase58())
    const mint = new web3.PublicKey("BTBoZDhtMcpTt8dRMcVD8xMu7E9tTCGQYns3K9g46tPY")

    const umi = createUmi("https://api.devnet.solana.com");
    const signer = createSignerFromKeypair(umi , fromWeb3JsKeypair(myKeypair) );
    umi.use(signerIdentity(signer , true));

    const accounts :  CreateMetadataAccountV3InstructionAccounts = {
        mint: fromWeb3JsPublicKey(mint),
        mintAuthority : signer,

        
    }

    const OnChainData = {
        name: "BTB Finance",
        symbol: "BTB",
        uri: "https://bxncwuikt4mbybsdkmwk2d76mfuip3o2tdewmxu7e2zxtkou2aka.arweave.net/DdorUQqfGBwGQ1MsrQ_-YWiH7dqYyWZenyazeanU0BQ",
        // we don't need that
        sellerFeeBasisPoints: 0,
        creators: null,
        collection: null,
        uses: null
    }


    const data : CreateMetadataAccountV3InstructionDataArgs = {
        isMutable :  true,
        collectionDetails : null,
        data: OnChainData,

    }

    const txid = await createMetadataAccountV3(umi , {...accounts , ...data}).sendAndConfirm(umi);
    console.log(txid);



}

main()

