Below is the updated documentation with a fourth requirement added for the Orchestrator. This new section explains that the Orchestrator must be running with the write canister ID and the JOB_SCHEDULE set to run every 5 seconds for the best results.

---

# Benchmarking Setup

This project can be used to benchmark requests. Before you begin testing, you must ensure that three Internet Computer (IC) canisters, a notary server, and an Orchestrator are up and running.

## Notary Server

- **Default URL:**  
  Your local verity notary will be used by default at `http://localhost:8080`.  
- **Custom URL:**  
  Change this by setting the environment variable:  
  ```bash
  export NOTARY_URL="https://your-notary-url"
  ```

## Required Canisters & Orchestrator

The following must be deployed and running in order:

1. **Managed Verifier**  
   Repository: [Managed Verifier](https://github.com/usherlabs/verity-dp/tree/main/ic/managed/verifier)  
   *Note:* The canister ID for the Managed Verifier will be required in the next step.

2. **ADC Processor**  
   Location: [ADC Processor](../processor/ic)  
   To configure this canister, set the verifier address by calling the `set_verifier_canister` method.

3. **ADC Caller Example**  
   Run the ADC_Caller example and set the ADC address using the `set_adc_address` method.

4. **Orchestrator**  
   The Orchestrator should be running from the `./orchestrator` directory. Make sure to launch it with the following requirements:
   - **Write Canister ID:** Provide the write canister ID as required.
   - **Job Schedule:** Set the environment variable `JOB_SCHEDULE` to `*/5 * * * * *` to schedule the job to run every 5 seconds for optimal benchmarking results.
   
   For example, you might start the orchestrator as follows:
   ```bash
   export JOB_SCHEDULE="*/5 * * * * *"
   ./orchestrator --write-canister-id <your_write_canister_id>
   ```

## Running the Benchmark

Once all three canisters and the orchestrator are running, you can edit the benchmark URL (located in `./__tests__/test_adc.test.ts`) if needed. Then, prepare and run your tests using either npm or yarn:

```bash
# Using npm:
npm run prep
npm run test

# Or using yarn:
yarn prep
yarn test
```

### Our Benchmark

| Component    | Cycle Usage   |
| ------------ | ------------- |
| ADC CALLER   | 2,065,757     |
| ADC          | 21,104,227    |
| VERIFIER     | 98,669,250    |
| **Total**    | **121,839,234** |

### HTTP_OUTCALL Benchmark

| Metric                                      | Cycle Usage         |
| ------------------------------------------- | ------------------- |
| Execution HTTPS_OUT_CALL time               | 4818 ms             |
| HTTPS_OUT_CALL ADC CALLER cycle used        | 1,605,497,196       |
| HTTPS_OUT_CALL ADC cycle used               | 101,455             |
| HTTPS_OUT_CALL VERIFIER cycle used          | 106,086             |
| **Total HTTPS_OUT_CALL cycle used**         | **1,605,704,737**   |

---

# processor_caller Project

Welcome to your new `processor_caller` project and the Internet Computer development community! When you create a project, this README and a set of template files are added automatically to help speed up your development cycle. Feel free to customize these files to suit your needs.

## Getting Started

Take a moment to explore the project directory and review the default configuration file. Changes made locally will not affect any production deployment or identity tokens.

For more detailed guidance, consult the following resources:

- [Quick Start](https://internetcomputer.org/docs/current/developer-docs/setup/deploy-locally)
- [SDK Developer Tools](https://internetcomputer.org/docs/current/developer-docs/setup/install)
- [Rust Canister Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [ic-cdk Documentation](https://docs.rs/ic-cdk)
- [ic-cdk-macros Documentation](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)

If you're ready to begin, try running the following commands:

```bash
cd processor_caller/
dfx help
dfx canister --help
```

## Running the Project Locally

To test your project on your local machine, follow these steps:

1. **Start the Local Replica:**

   ```bash
   dfx start --background
   ```

2. **Deploy the Canisters:**

   ```bash
   dfx deploy
   ```

   After deployment, your application will be available at:
   ```
   http://localhost:4943?canisterId={asset_canister_id}
   ```

3. **Regenerate the Candid Interface:**

   If you update your backend canister, run:
   ```bash
   npm run generate
   ```
   This step is recommended before starting the frontend development server and is also executed automatically during `dfx deploy`.

4. **Start the Frontend Development Server:**

   ```bash
   npm start
   ```
   This command starts a server at `http://localhost:8080` that proxies API requests to the replica on port 4943.

## Frontend Environment Variables

If you are hosting the frontend code without using DFX, consider the following adjustments to prevent your project from fetching the root key in production:

- **For Webpack Users:**  
  Set `DFX_NETWORK` to `ic`.

- **Replacing Environment Variables:**  
  Use your preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations. For example, in `dfx.json` you can set:
  ```json
  "canisters": {
    "asset_canister_id": {
      "declarations": {
        "env_override": "your_value"
      }
    }
  }
  ```

- **Custom Actor Creation:**  
  Alternatively, implement your own `createActor` constructor.

---

This updated documentation now includes a fourth requirement ensuring that the Orchestrator is running with the correct configuration for optimal benchmarking results. Happy coding!