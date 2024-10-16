/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/vesting.json`.
 */
export type Vesting = {
  "address": "4xxenLhJdt7ym5QDdep2RCv1qx9MyopPoLUzi7nC7YXL",
  "metadata": {
    "name": "vesting",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "fetchSaleAccount",
      "discriminator": [
        196,
        142,
        5,
        192,
        202,
        80,
        162,
        114
      ],
      "accounts": [
        {
          "name": "saleAccount"
        }
      ],
      "args": []
    },
    {
      "name": "initialize",
      "discriminator": [
        175,
        175,
        109,
        31,
        13,
        152,
        155,
        237
      ],
      "accounts": [
        {
          "name": "saleAccount",
          "writable": true,
          "signer": true
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "usdt",
          "type": "pubkey"
        },
        {
          "name": "btbtoken",
          "type": "pubkey"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "saleAccount",
      "discriminator": [
        213,
        18,
        87,
        228,
        218,
        230,
        207,
        182
      ]
    }
  ],
  "types": [
    {
      "name": "saleAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "usdt",
            "type": "pubkey"
          },
          {
            "name": "btbtoken",
            "type": "pubkey"
          }
        ]
      }
    }
  ]
};
