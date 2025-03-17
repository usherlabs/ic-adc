import { Actor, HttpAgent } from "@dfinity/agent";
import fetch from "isomorphic-fetch";
import adc_canisterIds from "../../processor/ic/.dfx/local/canister_ids.json";
import canisterIds from "../.dfx/local/canister_ids.json";

import { idlFactory as processor_idl } from "../../processor/ic/src/declarations/adc/adc.did.js";
import { idlFactory as adc_caller_idl } from "../src/declarations/adc_caller/adc_caller.did.js";

import { execSync } from "node:child_process";
import { identity } from "./identity.ts";

export function getCanisterCycles(canisterName: string): number {
  try {
    const statusOutput = execSync(`dfx canister status ${canisterName}`, {
      encoding: "utf-8",
    });
    const match = statusOutput.match(/Balance:\s+([\d_]+)/);
    if (match) {
      return Number.parseInt(match[1].replace(/_/g, ""), 10);
    }
  } catch (error) {
    console.error(`Error fetching canister cycles: ${error}`);
  }
  return 0;
}

export const createActor = async (canisterId, options, idl) => {
  const agent = new HttpAgent({ ...options?.agentOptions });
  const x = await agent.fetchRootKey();

  // Creates an actor with using the candid interface and the HttpAgent
  return Actor.createActor(idl, {
    agent,
    canisterId,
    ...options?.actorOptions,
  });
};

export const adc_caller = canisterIds.adc_caller.local;
export const processor_canister = adc_canisterIds.adc.local;

export const ADC_CALLER = await createActor(
  adc_caller,
  {
    agentOptions: {
      host: "http://127.0.0.1:4943",
      fetch,
      identity: await identity,
    },
  },
  adc_caller_idl,
);

export const PROCESSOR_CALLER = await createActor(
  processor_canister,
  {
    agentOptions: {
      host: "http://127.0.0.1:4943",
      fetch,
      identity: await identity,
    },
  },
  processor_idl,
);
