#!/usr/bin/env python3
"""
Test GPT model API connections using keys from all_vendor_api_keys.json.
Tests each OpenAI-format vendor and reports connectivity results.
"""
import json, time, urllib.request, urllib.error

API_KEYS_PATH = "C:/deep-student/scripts/all_vendor_api_keys.json"

def load_vendors(path):
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    return data["vendors"], data.get("test_config", {})

def test_vendor(name, vendor_info, config):
    base_url = vendor_info["base_url"].rstrip("/")
    api_key = vendor_info.get("api_key", "")
    models = vendor_info.get("test_models", [])
    if not api_key or api_key.startswith("PLACEHOLDER_"):
        return {"vendor": name, "display_name": vendor_info.get("name", name),
                "status": "SKIPPED", "reason": "No real API key configured", "models_tested": []}
    test_message = config.get("test_message", "Hi, please respond with a single word: connected")
    max_tokens = config.get("max_tokens", 10)
    timeout = config.get("timeout_seconds", 30)
    results = []
    for model in models:
        endpoint = f"{base_url}/chat/completions"
        payload = json.dumps({"model": model,
                              "messages": [{"role": "user", "content": test_message}],
                              "max_tokens": max_tokens, "stream": False}).encode("utf-8")
        req = urllib.request.Request(endpoint, data=payload,
                                     headers={"Content-Type": "application/json",
                                              "Authorization": f"Bearer {api_key}"},
                                     method="POST")
        mr = {"model": model, "endpoint": endpoint, "http_status": None,
              "latency_ms": None, "error": None, "success": False, "response_preview": None}
        try:
            start = time.time()
            resp = urllib.request.urlopen(req, timeout=timeout)
            elapsed = int((time.time() - start) * 1000)
            mr["latency_ms"] = elapsed
            mr["http_status"] = resp.status
            body = resp.read().decode("utf-8")
            data = json.loads(body)
            if "choices" in data and len(data["choices"]) > 0:
                content = data["choices"][0].get("message", {}).get("content", "")
                mr["success"] = True
                mr["response_preview"] = content[:80]
            elif "error" in data:
                mr["error"] = data["error"].get("message", str(data["error"]))
            else:
                mr["error"] = f"Unexpected response: {list(data.keys())}"
                mr["response_preview"] = json.dumps(data)[:200]
        except urllib.error.HTTPError as e:
            elapsed = int((time.time() - start) * 1000)
            mr["http_status"] = e.code
            mr["latency_ms"] = elapsed
            try:
                err_body = e.read().decode("utf-8", errors="replace")
                err_data = json.loads(err_body)
                mr["error"] = err_data.get("error", {}).get("message", err_body[:300])
            except Exception:
                mr["error"] = f"HTTP {e.code}: {str(e)[:200]}"
        except urllib.error.URLError as e:
            elapsed = int((time.time() - start) * 1000)
            mr["latency_ms"] = elapsed
            mr["error"] = f"URLError: {str(e.reason)[:300]}"
        except Exception as e:
            mr["error"] = f"Exception: {str(e)[:300]}"
        results.append(mr)
    successes = [r for r in results if r["success"]]
    failures = [r for r in results if not r["success"]]
    http_errors = [r for r in results if r["http_status"] and r["http_status"] >= 400]
    if len(successes) == len(models):
        status = "OK"
    elif len(successes) > 0:
        status = "PARTIAL"
    elif http_errors:
        codes = [str(r["http_status"]) for r in http_errors]
        if "502" in codes:
            status = "FAIL_502"
        elif "401" in codes or "403" in codes:
            status = "FAIL_AUTH"
        else:
            status = f"FAIL_HTTP_{'_'.join(set(codes))}"
    else:
        status = "FAIL"
    return {"vendor": name, "display_name": vendor_info.get("name", name),
            "status": status, "base_url": base_url, "models_tested": results}

def main():
    border = "=" * 72
    print(border)
    print("  GPT Model API Connectivity Test")
    print(border)
    print(f"  Keys source: {API_KEYS_PATH}")
    print(f"  Time: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(border)
    vendors, config = load_vendors(API_KEYS_PATH)
    vendor_order = ["siliconflow", "deepseek", "qwen", "zhipu", "doubao", "moonshot", "minimax", "mimo"]
    all_results = {}
    summary = []
    for vendor_key in vendor_order:
        if vendor_key not in vendors:
            continue
        vendor_info = vendors[vendor_key]
        name = vendor_info.get("name", vendor_key)
        print(f"\n{'-' * 72}")
        print(f"  [{vendor_key}] {name}")
        print(f"  URL: {vendor_info['base_url']}")
        print(f"{'-' * 72}")
        result = test_vendor(vendor_key, vendor_info, config)
        all_results[vendor_key] = result
        for mr in result["models_tested"]:
            icon = "[OK]" if mr["success"] else "[FAIL]"
            status_str = f"HTTP {mr['http_status']}" if mr["http_status"] else "No response"
            latency = f"{mr['latency_ms']}ms" if mr["latency_ms"] else "N/A"
            err = f" -- {mr['error'][:100]}" if mr["error"] else ""
            preview = f" | Resp: {mr['response_preview'][:60]}" if mr["response_preview"] else ""
            print(f"    {icon} {mr['model']:40s} {status_str:12s} {latency:10s}{err}{preview}")
        summary.append((vendor_key, result["status"], result["display_name"]))
    print(f"\n{border}")
    print("  SUMMARY")
    print(border)
    print(f"  {'Vendor':<25s} {'Status':<15s} {'Result'}")
    print(f"  {'-'*25} {'-'*15} {'-'*30}")
    for key, status, name in summary:
        sd = {"OK": "[OK]", "PARTIAL": "[~] PARTIAL", "FAIL_502": "[X] FAIL (502)",
              "FAIL_AUTH": "[X] FAIL (Auth)", "FAIL": "[X] FAIL", "SKIPPED": "[-] SKIPPED"}.get(status, status)
        verdicts = {"OK": "Connected successfully", "PARTIAL": "Some models worked, some failed",
                    "FAIL_502": "*** 502 Bad Gateway -- proxy likely broken ***",
                    "FAIL_AUTH": "Authentication failed -- check API key",
                    "FAIL": "Failed -- check URL and connectivity"}
        verdict = verdicts.get(status, f"HTTP error ({status})") if "FAIL_HTTP" not in status else f"HTTP error ({status})"
        print(f"  {name:<25s} {sd:<15s} {verdict}")
    print(f"\n{border}")
    return all_results

if __name__ == "__main__":
    results = main()
    summary_out = {}
    for k, v in results.items():
        summary_out[k] = {"status": v["status"], "display_name": v["display_name"],
                          "models": [{"model": m["model"], "success": m["success"],
                                      "http_status": m["http_status"], "latency_ms": m["latency_ms"],
                                      "error": m["error"]} for m in v["models_tested"]]}
    print("\n---MACHINE_JSON---")
    print(json.dumps(summary_out, ensure_ascii=False, indent=2))
