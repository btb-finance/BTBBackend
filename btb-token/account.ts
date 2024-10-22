/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/account.json`.
 */
export type Account = {
  "address": "52r2huu9PiWxebKNhwcmE2WohXXDbupssVLBDFS8zVnH",
  "metadata": {
    "name": "account",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "fetchAccount",
      "discriminator": [
        194,
        55,
        239,
        245,
        231,
        248,
        6,
        125
      ],
      "accounts": [
        {
          "name": "investment",
          "writable": true
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
          "name": "investment",
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
          "name": "name",
          "type": "string"
        },
        {
          "name": "balance",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "investment",
      "discriminator": [
        175,
        134,
        9,
        175,
        115,
        153,
        39,
        28
      ]
    }
  ],
  "types": [
    {
      "name": "investment",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "balance",
            "type": "u64"
          }
        ]
      }
    }
  ]
};
