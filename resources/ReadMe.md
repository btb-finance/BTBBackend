
cd BTBBackendV1/resources/

# Create a new Keypair 
`solana-keygen grind --starts-with  <custom starting letters like BTB>:<number of keys>`

`solana config set --keypair BTBF9x3R2nnY1cvP32KPjeyE26XQV8z7Y86xtPydxaRk.json`

`solana config set --url devnet`

`solana airdrop 5`

# now create token Program Account 

`spl-token create-token --program-id TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb --enable-metadata mnti2XLiWJ2H2YaLdCDX3Js2c6AdCxZ2FMUuhV2abEy.json`


To initialize metadata inside the mint, please run 

`spl-token initialize-metadata mnti2XLiWJ2H2YaLdCDX3Js2c6AdCxZ2FMUuhV2abEy <YOUR_TOKEN_NAME> <YOUR_TOKEN_SYMBOL> <YOUR_TOKEN_URI>`

Sample 

`spl-token initialize-metadata mnti2XLiWJ2H2YaLdCDX3Js2c6AdCxZ2FMUuhV2abEy "Sample Token desc" "SYMBOL" https://emerald-past-falcon-624.mypinata.cloud/ipfs/QmbRGMLeRMjpFSTjuo7EYGC6puCCZsfa72kSgJ3TS7dwNL`,
 and sign with the mint authority.

--- till this step the token has been created and the metadata has been added to it, now it is the time to create
some tokens ------

# Now create the sale account for mint program

`spl-token create-account mnti2XLiWJ2H2YaLdCDX3Js2c6AdCxZ2FMUuhV2abEy`

# at this point, we have two accounts
    # 1. Token Program Account
    # 2. Token Sale Account 

# Now create 100 tokens to the account 

`spl-token mint mnti2XLiWJ2H2YaLdCDX3Js2c6AdCxZ2FMUuhV2abEy 100 `

# You can transfer token with the help of below command 

`spl-token transfer mnti2XLiWJ2H2YaLdCDX3Js2c6AdCxZ2FMUuhV2abEy 10 (recipient wallet address) --fund-recipient`


