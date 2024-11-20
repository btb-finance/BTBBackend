# add bankrun dependencies

`yarn add solana-bankrun anchor-bankrun`



# setting up a new contract 

1. build the project
2. call the initialize method in client folder to inject demo values in newly deployed SC
3. call the update_initialize method to add the tokens from  demo admin
    3.1  solana config set -k owner_signer_wallet.json  
    3.2  spl-token transfer btbVv5dmAjutpRRSr6DKwBPyPyfKiJw4eXU11BPuTCK 10000 6BBErfPWbhZBqLic2uoh8PqoktVDb5XuTe4z69vzDcui