##  Benchmarking Workflow

Before running your tests, ensure that all required services and canisters are deployed and running. The workflow is split into four main phases:

### 1. Install Dependencies

Make sure all project dependencies are installed:
```bash
yarn install
```

---

### 2. Prepare the Environment

Run the preparation script to configure your local environment and generate any required artifacts:

```bash
yarn prep
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

### PROVER Server
- **Default URL:**  
  The local verity prover is assumed to be running at `http://localhost:8080`.

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
