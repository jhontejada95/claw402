import assert from "node:assert/strict";
import test from "node:test";

async function render(path = "/") {
  const workerUrl = new URL("../dist/server/index.js", import.meta.url);
  workerUrl.searchParams.set("test", `${process.pid}-${Date.now()}-${path}`);
  const { default: worker } = await import(workerUrl.href);

  return worker.fetch(
    new Request(`http://localhost${path}`, {
      headers: { accept: "text/html" },
    }),
    {
      ASSETS: {
        fetch: async () => new Response("Not found", { status: 404 }),
      },
    },
    {
      waitUntil() {},
      passThroughOnException() {},
    },
  );
}

async function htmlFor(path) {
  const response = await render(path);
  assert.equal(response.status, 200, `${path} should render successfully`);
  assert.match(response.headers.get("content-type") ?? "", /^text\/html\b/i);
  return response.text();
}

test("server-renders the commercial Claw402 homepage", async () => {
  const html = await htmlFor("/");

  assert.match(html, /Let your agents buy/i);
  assert.match(html, /Keep your wallet locked/i);
  assert.match(html, /Devnet procurement is live/i);
  assert.match(html, /Install the firewall/i);
  assert.doesNotMatch(html, /Your site is taking shape|Building your site/i);
});

test("product page reports shipped devnet settlement honestly", async () => {
  const html = await htmlFor("/product");

  assert.match(html, /Restricted devnet payments/i);
  assert.match(html, /x402 settlement/i);
  assert.match(html, /Production hardening/i);
  assert.match(html, /mainnet settlement are intentionally not enabled/i);
});

test("developer and security pages preserve the custody boundary", async () => {
  const [developers, security] = await Promise.all([
    htmlFor("/developers"),
    htmlFor("/security"),
  ]);

  assert.match(developers, /verified receipts/i);
  assert.match(developers, /mainnet settlement remains intentionally disabled/i);
  assert.match(security, /restricted signer/i);
  assert.match(security, /devnet/i);
  assert.match(security, /private keys/i);
});
