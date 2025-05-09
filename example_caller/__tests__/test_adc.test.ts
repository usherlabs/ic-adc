import { Principal } from "@dfinity/principal";
import axios from "axios";
import { beforeAll, describe, expect, it } from "vitest";
import { ADC_CALLER, PROCESSOR_CALLER, adc_caller, getCanisterCycles, processor_canister } from "./actor";

// biome-ignore lint/style/useSingleVarDeclarator: initialized variable
let verifier: string, total_http_out_call: number;

const target_url = "https://api-itnet.nearblocks.io/v1/account/x-bitte-nfts.testnet/txns-only?cursor=0&order=asc";
const method = "GET";
const redacted = "";
const headers = [];
const body = "";

function wait(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

describe("ADC-caller IC Verifier", () => {
  beforeAll(async () => {
    const notary_info = await axios.get(`${process.env.PROVER_URL ?? "http://127.0.0.1:8080"}/notaryinfo`);
    expect(notary_info.status, "Notary is no active").toBe(200);
    expect(notary_info.data.publicKey.startsWith("-----BEGIN PUBLIC KEY-----")).toBe(true);

    //expect adc_address to be set
    const result = (await PROCESSOR_CALLER.get_verifier_canister()) as any;
    expect(result).toBeDefined();
    verifier = Principal.fromUint8Array(result[0]._arr).toText();
  });

  it("expect set adc_address", async () => {
    const result = (await ADC_CALLER.get_adc_address()) as any;
    expect(result).toBeDefined();
    expect(Principal.fromUint8Array(result[0]._arr).toText()).toBe(processor_canister);
  });

  it("expect set verifier", async () => {
    const result = (await PROCESSOR_CALLER.get_verifier_canister()) as any;
    expect(result).toBeDefined();
    expect(Principal.fromUint8Array(result[0]._arr).toText().length, "verifier must be set").toBe(27);
  });

  it("estimate cost for HTTPS_OUT_CALL", async () => {
    const old_balance_adc_caller = await getCanisterCycles(adc_caller);
    const old_balance_adc = await getCanisterCycles(processor_canister);
    const old_balance_verifier = await getCanisterCycles(verifier);

    const startTime = Date.now();
    const result = (await ADC_CALLER.send_http_request(target_url, method, redacted, headers, body)) as any;
    expect(result).toBeDefined();
    expect(result).toContain("txns");
    console.log(`Execution HTTPS_OUT_CALL time: ${Date.now() - startTime} ms`);
    const _adc_caller = old_balance_adc_caller - (await getCanisterCycles(adc_caller));
    const _adc = old_balance_adc - (await getCanisterCycles(processor_canister));
    const _verifier = old_balance_verifier - (await getCanisterCycles(verifier));
    console.log("HTTPS_OUT_CALL ADC CALLER cycle used:\t\t", _adc_caller);
    console.log("HTTPS_OUT_CALL ADC    cycle used:\t\t", _adc);
    console.log("HTTPS_OUT_CALL VERIFIER cycle used:\t\t", _verifier);
    console.log("TOTAL HTTPS_OUT_CALL  cycle used:\t\t", _adc_caller + _adc + _verifier);
    total_http_out_call = _adc_caller + _adc + _verifier;
  }, 60000);

  it("estimate cost for our_http_orchestrator", async () => {
    const old_balance_adc_caller = await getCanisterCycles(adc_caller);
    const old_balance_adc = await getCanisterCycles(processor_canister);
    const old_balance_verifier = await getCanisterCycles(verifier);

    const startTime = Date.now();
    const request_id = (await ADC_CALLER.submit_http_request(target_url, method, redacted, headers, body)) as any;
    expect(request_id).toBeDefined();
    console.log(`Execution time: ${Date.now() - startTime} ms`);
    console.log(`Execution HTTPS_OUT_CALL time: ${Date.now() - startTime} ms`);
    await wait(60000);
    const result = (await ADC_CALLER.get_adc_response(request_id)) as any;
    expect(result.length).toBe(1);
    expect(result[0]).toContain("txns");
    expect(result[0].length).toBeGreaterThan(2024);
    const _adc_caller = old_balance_adc_caller - (await getCanisterCycles(adc_caller));
    const _adc = old_balance_adc - (await getCanisterCycles(processor_canister));
    const _verifier = old_balance_verifier - (await getCanisterCycles(verifier));
    console.log("ADC CALLER cycle used:\t\t", _adc_caller);
    console.log("ADC cycle   used:\t\t", _adc);
    console.log("VERIFIER cycle used:\t\t", _verifier);
    console.log("TOTAL cycle used:\t\t", _adc_caller + _adc + _verifier);

    console.log(
      `Cost savings    :\t\t${((1 - (_adc_caller + _adc + _verifier) / total_http_out_call) * 100).toFixed(2)} %`,
    );
  }, 1000000);
});
