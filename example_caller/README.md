# IC-ADC `example_caller`

The `example_caller` demonstrates how to call the Asset Data Canister (ADC) from within another Canister via XNET (inter-canister calls).

## Integrating with the IC-ADC

We recommend cloning this repository and extracting the `adc_caller` Canister into your own project to establish a foundation for integrating and interacting with the IC-ADC within the Internet Computer.

```shell
# Clone the IC-ADC repository
git clone https://github.com/usherlabs/ic-adc

# Copy the adc_caller files to your project
# Replace YOUR_PROJECT_PATH with your actual project path
cp -r ic-adc/example_caller/src/adc_caller/ YOUR_PROJECT_PATH/

# Modify the dfx.json within your project accordingly
vim YOUR_PROJECT_PATH/dfx.json
```

As per the `./src/adc_caller/adc_caller.did`, there are two distinct methods for requesting data from third-party data sources:

- `submit_http_request` — ckTLS
- `send_http_request` — HTTPS Outcalls

`ckTLS` leverages the Verity Network and Data Processor Framework to generate MPC-TLS proofs, and verify them within the IC Canister. 

Once you've cloned the package into your project, you can prune the methods that you do not require.
If you're using `submit_http_request` (ckTLS) for high-frequency low-cost data indexing on the IC, it's unlikely you will need HTTPS Outcalls.

## Benchmarking Workflow

The `example_caller` also serves as a foundation for a benchmarking suite that showcases the effectiveness of ckTLS 🧪 (MPC-TLS) compared with the IC's native HTTPS Outcalls.

Before running your tests, ensure all required services and canisters are deployed and running. The workflow is divided into four main phases:

### 1. Install Dependencies

Make sure all project dependencies are installed:

```bash
pnpm install
```

---

### 2. Prepare the Environment

Run the preparation script to configure your local environment and generate any required artefacts:

```bash
pnpm prep
```

---

### 3. Deploy Required Canisters and Start the Orchestrator

You must deploy several Internet Computer canisters (from a foreign repository) and start the Orchestrator. Follow these substeps:

- **Deploy Canisters in Order:**

  1. **Managed Verifier:**  
     - Repository: [Managed Verifier](https://github.com/usherlabs/verity-dp/tree/main/ic/managed/verifier)  
     - *Note:* You'll need the Managed Verifier's canister ID for later configuration.

  2. **ADC Processor:**  
     - Location: [ADC Processor](../processor/ic)  
     - After deployment, configure it by calling the `set_verifier_canister` method with the Managed Verifier's canister ID.

  3. **ADC Caller Example:**  
     - Deploy and configure it by calling the `set_adc_address` method with the appropriate ADC address.

- **Start the Orchestrator:**  
  Navigate to the `./orchestrator` directory and run the orchestrator with the following steps:

  1. Set the job schedule (to run every 5 seconds for optimal benchmarking):
  
     ```bash
     export JOB_SCHEDULE="*/2 * * * * *"
     ```
     
  2. Launch the orchestrator by providing the write canister ID:

     ```bash
     cd ./orchestrator && cargo run
     ```

---

### 4. Run the Tests

After ensuring all services are up, run your tests:

```bash
yarn test
```

---

## Additional Setup Details

### Verity Prover Server

- **Default URL:**  
  The local Verity Prover is assumed to be running at `http://localhost:8080`.

- **Custom URL:**  
  If using a different URL, export the following environment variable:
  
  ```bash
  export PROVER_URL="https://your-prover-url"
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

By following these steps in order, you ensure that your environment is properly configured and all necessary services are running before executing your tests. This clear workflow minimizes errors and improves reproducibility.
