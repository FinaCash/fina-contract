# time-lock contract for snip token

This is an implementation of a linear vesting contract for [SNIP-20](https://docs.scrt.network/secret-network-documentation/development/snips/snip-20-spec-private-fungible-tokens) token. The vesting calculation is up to second precision. And query is available to provide transparency on the vesting progress and schedule.

## Troubleshooting 

All transactions are encrypted, so if you want to see the error returned by a failed transaction, you need to use the command

`secretcli q compute tx <TX_HASH>`