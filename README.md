# Internet Computer Asset Data Canister (ADC)

The ADC is a highly efficient Oracle for real-world and cryptocurrency asset data on the Internet Computer. Developed by [Usher Labs](https://www.usher.so) and powered by the Verity Protocol.

The unique Chain-key architecture of the IC allows for new ways of combining off-chain data sourcing with on-chain computation, resulting in concise proofs of entire data flows. This approach addresses the challenges of traditional methods, such as the high costs and technical difficulties of HTTPS Outcalls for fetching data, especially from foreign blockchains. Additionally, there is currently no standardised framework for efficiently integrating foreign blockchain data, particularly for digital asset management and verifying private API data due to security concerns.

The ADC serves as an example of using the Internet Computer (IC) as a decentralised co-processor.

## Data Supported & Roadmap

- [x] Cryptocurrency Asset Prices
- [ ] Cryptocurrency Asset Market Capitalisations
- [ ] Cryptocurrency Asset Volume
- [ ] Cryptocurrency Asset Uniswap Liquidity
- [ ] Cryptocurrency Asset Token Holder Count
- [ ] Arbitrary EVM Blockchain Data
- [ ] IoT Data (Streamr & Helium)

## Getting Started

The ADC works similarly to the native [IC Exchange Rate Canister](https://github.com/internetcomputer/exchange-rate-canister). Calling Canisters integrate with the ADC by making [XNET calls](https://internetcomputer.org/how-it-works/message-routing/) and waiting for a response from the ADC.

You can find an example of this implementation in the [`example_caller`](./example_caller) folder.

:::note 
Currently, your calling canister must be whitelisted. To get whitelisted, [please contact Usher Labs directly](https://www.usher.so/contact/), or [join our Discord](https://go.usher.so/discord).
:::

### Overview

The ADC operates on a pull model. Requests are sent to the ADC for specific data, and an off-chain `orchestrator` process handles making and cryptographically proving these requests. The ADC then verifies and forwards the processed or unprocessed result to the Calling Canister.

**There are two main actions when interacting with the ADC:**

1. **Requesting price details about a currency pair:**

When you make a request, you receive an `id` that you can use to track the response. This `id` corresponds to the timestamp when the request was received by the processor.

```rust
#[ic_cdk::update]
/// where `currency_pairs` is a comma seperated list of pairs
/// e.g "BTC,ETH/USDT,sol"
async fn submit_adc_request(currency_pairs: String) -> String {
    let adc_canister_request_method = "request_data";
    //TODO: change the principal to that of the processor's
    let processor_canister_principal = Principal::from_str("bkyz2-fmaaa-aaaaa-qaaaq-cai").unwrap();
    let options = RequestOpts::default();

    let (request_id,): (String,) = ic_cdk::call(
        processor_canister_principal,
        adc_canister_request_method,
        (currency_pairs, options, ),
    )
    .await
    .unwrap();

    // println!("{:?}", request_id)
    return request_id;
}
```

2. **Receiving a response for a request:**

To receive a response, an `update` function called `receive_adc_response` must be present on the Calling Canister.

```rust

#[ic_cdk::update]
fn receive_adc_response(response: ADCResponse) {
    println!("receive_adc_response: {:?}", response);
}
```

To reiterate, a working example of a Calling Canister with the relevant schema can be found in [this example](./example_caller).
