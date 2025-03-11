import { Principal } from "@dfinity/principal";
import { beforeAll, describe, expect, test, Vitest, vitest } from "vitest";
import { getCanisterCycles, ADC_CALLER, PROCESSOR_CALLER,processor_canister, adc_caller } from "./actor";
import axios from "axios"




let verifier:string;

const target_url="https://api-testnet.nearblocks.io/v1/account/x-bitte-nfts.testnet/txns-only?cursor=0&order=asc";
const method="GET"
const redacted=""
const headers=[]
const body=""


function wait(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe("ADC-caller IC Verifier", () => {

  beforeAll(async()=>{
    const notary_info = await axios.get(`${process.env.NOTARY_URL??"http://127.0.0.1:8080"}/notaryinfo`)
    expect(notary_info.status,"Notary is no active").toBe(200)
    expect(notary_info.data.publicKey.startsWith("-----BEGIN PUBLIC KEY-----")).toBe(true)

    //expect adc_address to be set
    const result = await PROCESSOR_CALLER.get_verifier_canister() as any;
    expect(result).toBeDefined();
    verifier=Principal.fromUint8Array(result[0]._arr).toText();
  })

  test("expect set adc_address", async () => {
    const result = await ADC_CALLER.get_adc_address() as any;
    expect(result).toBeDefined();
    expect(Principal.fromUint8Array(result[0]._arr).toText()).toBe(processor_canister)
  });

  test("expect set verifier", async () => {
    const result = await PROCESSOR_CALLER.get_verifier_canister() as any;
    expect(result).toBeDefined();
    expect(Principal.fromUint8Array(result[0]._arr).toText().length,"verifier must be set").toBe(27)
  });

  test("estimate cost for HTTPS_OUT_CALL", async () => {
    const old_balance_adc_caller=await getCanisterCycles(adc_caller)
    const old_balance_adc=await getCanisterCycles(processor_canister)
    const old_balance_verifier=await getCanisterCycles(verifier)

    const startTime = Date.now();
    const result = await ADC_CALLER.send_http_request(target_url,method,redacted,headers,body) as any;
    expect(result).toBeDefined();
    expect(result).toContain("txn")
    console.log(`Execution HTTPS_OUT_CALL time: ${Date.now()-startTime} ms`);
    const _adc_caller=old_balance_adc_caller-await getCanisterCycles(adc_caller)
    const _adc = old_balance_adc-(await getCanisterCycles(processor_canister))
    const _verifier = old_balance_verifier-(await getCanisterCycles(verifier))
    console.log("HTTPS_OUT_CALL ADC CALLER cycle used:",_adc_caller)
    console.log("HTTPS_OUT_CALL ADC cycle used:",_adc)
    console.log("HTTPS_OUT_CALL VERIFIER cycle used:",_verifier)
    console.log("TOTAL HTTPS_OUT_CALL cycle used:",_adc_caller+_adc+_verifier)
  },60000);


  test("estimate cost for our_http_orchestrator", async () => {
    const old_balance_adc_caller=await getCanisterCycles(adc_caller)
    const old_balance_adc=await getCanisterCycles(processor_canister)
    const old_balance_verifier=await getCanisterCycles(verifier)

    const startTime = Date.now();
    const request_id = await ADC_CALLER.submit_http_request(target_url,method,redacted,headers,body) as any;
    expect(request_id).toBeDefined();
    console.log(`Execution time: ${Date.now()-startTime} ms`);
    console.log(`Execution HTTPS_OUT_CALL time: ${Date.now()-startTime} ms`);
    await wait(60000);
    const result = await ADC_CALLER.get_adc_response(request_id) as any;
    expect(result.length).toBe(1);
    expect(result[0].length).toBeGreaterThan(2024);
    const _adc_caller=old_balance_adc_caller-await getCanisterCycles(adc_caller)
    const _adc = old_balance_adc-(await getCanisterCycles(processor_canister))
    const _verifier = old_balance_verifier-(await getCanisterCycles(verifier))
    console.log("ADC CALLER cycle used:",_adc_caller)
    console.log("ADC cycle used:",_adc)
    console.log("VERIFIER cycle used:",_verifier)
    console.log("TOTAL cycle used:",_adc_caller+_adc+_verifier)

  },1000000);

})