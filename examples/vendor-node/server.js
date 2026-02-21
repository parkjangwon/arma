const http = require("http");

const VENDOR_PORT = Number(process.env.VENDOR_PORT || 3000);
const ARMA_BASE_URL = process.env.ARMA_BASE_URL || "http://127.0.0.1:8080";
const ARMA_TIMEOUT_MS = Number(process.env.ARMA_TIMEOUT_MS || 500);
const ARMA_FAIL_MODE = process.env.ARMA_FAIL_MODE || "open";

function sendJson(res, statusCode, payload) {
  const body = JSON.stringify(payload);
  res.writeHead(statusCode, {
    "content-type": "application/json",
    "content-length": Buffer.byteLength(body),
  });
  res.end(body);
}

async function readJson(req) {
  return new Promise((resolve, reject) => {
    let data = "";
    req.on("data", (chunk) => {
      data += chunk;
    });
    req.on("end", () => {
      try {
        resolve(data ? JSON.parse(data) : {});
      } catch (error) {
        reject(error);
      }
    });
    req.on("error", reject);
  });
}

async function validateWithArma(prompt) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), ARMA_TIMEOUT_MS);

  try {
    const response = await fetch(`${ARMA_BASE_URL}/v1/validate`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ prompt, user_id: "vendor-sample-user" }),
      signal: controller.signal,
    });

    if (!response.ok) {
      throw new Error(`ARMA returned ${response.status}`);
    }

    return await response.json();
  } finally {
    clearTimeout(timeoutId);
  }
}

function mockLlm(prompt) {
  return `Mock LLM response for: ${prompt.slice(0, 120)}`;
}

const server = http.createServer(async (req, res) => {
  if (req.method === "GET" && req.url === "/health") {
    return sendJson(res, 200, {
      status: "ok",
      service: "vendor-node-sample",
      arma_base_url: ARMA_BASE_URL,
      arma_fail_mode: ARMA_FAIL_MODE,
    });
  }

  if (req.method === "POST" && req.url === "/chat") {
    let payload;
    try {
      payload = await readJson(req);
    } catch (_error) {
      return sendJson(res, 400, { error: "invalid_json" });
    }

    const prompt = typeof payload.prompt === "string" ? payload.prompt.trim() : "";
    if (!prompt) {
      return sendJson(res, 400, { error: "prompt_required" });
    }

    try {
      const validation = await validateWithArma(prompt);
      if (!validation.is_safe) {
        return sendJson(res, 403, {
          blocked: true,
          source: "ARMA",
          reason: validation.reason,
          score: validation.score,
        });
      }

      return sendJson(res, 200, {
        blocked: false,
        source: "ARMA",
        arma_bypassed: false,
        arma_reason: validation.reason,
        arma_score: validation.score,
        answer: mockLlm(prompt),
      });
    } catch (error) {
      if (ARMA_FAIL_MODE === "closed") {
        return sendJson(res, 503, {
          blocked: true,
          source: "ARMA",
          reason: "arma_unavailable_fail_closed",
          detail: String(error.message || error),
        });
      }

      return sendJson(res, 200, {
        blocked: false,
        source: "VENDOR_BYPASS",
        arma_bypassed: true,
        arma_error: String(error.message || error),
        answer: mockLlm(prompt),
      });
    }
  }

  return sendJson(res, 404, { error: "not_found" });
});

server.listen(VENDOR_PORT, "0.0.0.0", () => {
  console.log(`[vendor-node-sample] listening on 0.0.0.0:${VENDOR_PORT}`);
  console.log(`[vendor-node-sample] ARMA=${ARMA_BASE_URL}, FAIL_MODE=${ARMA_FAIL_MODE}`);
});
