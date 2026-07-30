/** Cloudflare Worker entry point for the vinext-starter template. */
import { handleImageOptimization, DEFAULT_DEVICE_SIZES, DEFAULT_IMAGE_SIZES } from "vinext/server/image-optimization";
import handler from "vinext/server/app-router-entry";

interface Env {
  ASSETS: Fetcher;
  DB: D1Database;
  IMAGES: {
    input(stream: ReadableStream): {
      transform(options: Record<string, unknown>): {
        output(options: { format: string; quality: number }): Promise<{ response(): Response }>;
      };
    };
  };
}

interface ExecutionContext {
  waitUntil(promise: Promise<unknown>): void;
  passThroughOnException(): void;
}

const X402_FACILITATOR = "https://x402.org/facilitator";
const SOLANA_DEVNET = "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1";
const USDC_DEVNET = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";
const DEMO_MERCHANT = "CepLpqTzeN4EWnVE9jFFv79DMG4cGHn6hxLh5cffWvJ";
const DEMO_FEE_PAYER = "CKPKJWNdJEqa81x7CkZ14BVPiY6y16Sxs7owznqtWYp5";

function paymentRequirement() {
  return {
    scheme: "exact",
    network: SOLANA_DEVNET,
    amount: "1000",
    payTo: DEMO_MERCHANT,
    maxTimeoutSeconds: 60,
    asset: USDC_DEVNET,
    extra: { feePayer: DEMO_FEE_PAYER },
  };
}

function paymentRequired(request: Request, error?: string): Response {
  const challenge = {
    x402Version: 2,
    resource: {
      url: new URL(request.url).origin + "/api/demo-rpc",
      description: "Claw402 policy-protected Solana devnet RPC demo",
      mimeType: "application/json",
    },
    accepts: [paymentRequirement()],
  };
  const headers = new Headers({
    "content-type": "application/json",
    "cache-control": "no-store",
    "PAYMENT-REQUIRED": btoa(JSON.stringify(challenge)),
  });
  return new Response(JSON.stringify({ ...challenge, error }), { status: 402, headers });
}

async function facilitatorCall(operation: "verify" | "settle", paymentPayload: unknown) {
  const response = await fetch(`${X402_FACILITATOR}/${operation}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      x402Version: 2,
      paymentPayload,
      paymentRequirements: paymentRequirement(),
    }),
  });
  const body = await response.json<Record<string, unknown>>();
  return { ok: response.ok, body };
}

async function handleDemoRpc(request: Request): Promise<Response> {
  if (request.method !== "POST") {
    return new Response("Method Not Allowed", {
      status: 405,
      headers: { allow: "POST", "cache-control": "no-store" },
    });
  }
  const contentLength = Number(request.headers.get("content-length") ?? "0");
  if (contentLength > 64 * 1024) {
    return new Response("Request Too Large", { status: 413 });
  }

  const paymentHeader = request.headers.get("PAYMENT-SIGNATURE");
  if (!paymentHeader) {
    return paymentRequired(request);
  }

  let paymentPayload: unknown;
  try {
    paymentPayload = JSON.parse(atob(paymentHeader));
  } catch {
    return paymentRequired(request, "Invalid PAYMENT-SIGNATURE encoding");
  }

  const verification = await facilitatorCall("verify", paymentPayload);
  if (!verification.ok || verification.body.isValid !== true) {
    const reason =
      String(verification.body.invalidReason ?? verification.body.invalidMessage ?? "verification rejected");
    return paymentRequired(request, reason);
  }

  const settlement = await facilitatorCall("settle", paymentPayload);
  if (!settlement.ok || settlement.body.success !== true) {
    const reason =
      String(settlement.body.errorReason ?? settlement.body.errorMessage ?? "settlement failed");
    return paymentRequired(request, reason);
  }

  let rpcRequest: Record<string, unknown> = {};
  try {
    rpcRequest = await request.json<Record<string, unknown>>();
  } catch {
    return new Response(JSON.stringify({ error: "invalid JSON-RPC body" }), {
      status: 400,
      headers: { "content-type": "application/json", "cache-control": "no-store" },
    });
  }

  return new Response(
    JSON.stringify({
      jsonrpc: "2.0",
      id: rpcRequest.id ?? null,
      result: {
        status: "ok",
        service: "Claw402 devnet demo provider",
        policy: "payment verified and settled before execution",
      },
    }),
    {
      status: 200,
      headers: {
        "content-type": "application/json",
        "cache-control": "no-store",
        "PAYMENT-RESPONSE": btoa(JSON.stringify(settlement.body)),
      },
    },
  );
}

// Image security config. SVG sources with .svg extension auto-skip the
// optimization endpoint on the client side (served directly, no proxy).
// To route SVGs through the optimizer (with security headers), set
// dangerouslyAllowSVG: true in next.config.js and uncomment below:
// const imageConfig: ImageConfig = { dangerouslyAllowSVG: true };

const worker = {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);

    if (url.pathname === "/api/demo-rpc") {
      return handleDemoRpc(request);
    }

    if (url.pathname === "/_vinext/image") {
      const allowedWidths = [...DEFAULT_DEVICE_SIZES, ...DEFAULT_IMAGE_SIZES];
      return handleImageOptimization(request, {
        fetchAsset: (path) => env.ASSETS.fetch(new Request(new URL(path, request.url))),
        transformImage: async (body, { width, format, quality }) => {
          const result = await env.IMAGES.input(body).transform(width > 0 ? { width } : {}).output({ format, quality });
          return result.response();
        },
      }, allowedWidths);
    }

    return handler.fetch(request, env, ctx);
  },
};

export default worker;
