# CLOBBY

Clobby is an Onchain Orderbook built on Solana.(Click on each image to view in full resolution)

![Image](https://github.com/user-attachments/assets/85ac2e80-5662-48ec-9619-e6686bafecb7)
![Image](https://github.com/user-attachments/assets/09a92651-3138-4e1d-8328-d04bd9f5417d)
![Image](https://github.com/user-attachments/assets/2c266bbe-59c8-401b-9909-440d92309986)

## Architecture
- Users create a *balance account* for each market, to claim their base and quote assets.
- Users Place a bid/ask order in  the market.
- The order will sit in the orderbook. The base/quote asset gets transferred from user account to market account.
- When an opposing order gets matched, the **fill events** and the **out events** are recorded in the `market_events` account.
- The market creator continously invokes `consume_events` instruction, which increases/decreases the base amount and quote amount of the maker/taker's balance account.
- Users can invoke `settle_balance` instruction, to get the assets from their balance account to their token account.


### Setup Guide
- Clone the Project.
- Run `anchor test` to start the tests.
- Build and Deploy the Program
		- ```anchor build```
		- ```anchor deploy```
- Run the tests
		- Navigate to the client folder ```cd client```
		- Replace the ProgramID in ```index.test.ts```
		- Run ```bun test --timeout 60000``` in the terminal to run the tests.



### CONTRIBUTIONS
If you feel an issue or something needs to be fixed , please raise an Issue or a PR. Your contributions are welcomed most !! :pray: