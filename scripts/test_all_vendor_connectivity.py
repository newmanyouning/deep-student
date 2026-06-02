#!/usr/bin/env python3
"""All Vendor Connectivity Test — reads all_vendor_api_keys.json, tests each vendor."""
import json, time, urllib.request, urllib.error, ssl, os, sys
from datetime import datetime, timezone

CONFIG_PATH = os.path.join(os.path.dirname(__file__) or ".", "all_vendor_api_keys.json")

def load_config():
    with open(CONFIG_PATH, "r", encoding="utf-8") as f:
        return json.load(f)

def api_request(method, url, headers, body=None, timeout=15):
    ctx = ssl.create_default_context()
    data = json.dumps(body).encode("utf-8") if body else None
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    start = time.monotonic()
    try:
        with urllib.request.urlopen(req, timeout=timeout, context=ctx) as resp:
            elapsed_ms = int((time.monotonic() - start) * 1000)
            raw = resp.read()
            return resp.status, json.loads(raw) if raw else {}, "", elapsed_ms
    except urllib.error.HTTPError as e:
        elapsed_ms = int((time.monotonic() - start) * 1000)
        try:
            body = json.loads(e.read())
        except:
            body = {"error": str(e)}
        return e.code, body, str(e), elapsed_ms
    except Exception as e:
        elapsed_ms = int((time.monotonic() - start) * 1000)
        return 0, {"error": str(e)}, str(e), elapsed_ms

def test_vendor(name, cfg, test_cfg):
    api_key = cfg.get("api_key", "")
    if not api_key or api_key.startswith("PLACEHOLDER_"):
        return {"vendor": name, "status": "skipped", "reason": "no API key"}

    base_url = cfg["base_url"].rstrip("/")
    headers = {"Authorization": f"Bearer {api_key}", "Content-Type": "application/json"}
    result = {"vendor": name, "base_url": base_url, "status": "ok", "models_test": {}, "timestamp": datetime.now(timezone.utc).isoformat()}

    # Step 1: Fetch model list
    status, data, err, elapsed = api_request("GET", f"{base_url}/models", headers)
    result["models_endpoint"] = {"ok": status == 200, "status": status, "elapsed_ms": elapsed, "error": err}
    if status == 200:
        api_models = [m.get("id", "") for m in data.get("data", []) if isinstance(m, dict)]
        result["models_endpoint"]["model_count"] = len(api_models)
        result["models_endpoint"]["models"] = api_models[:50]

    # Step 2: Test each model
    for model in cfg.get("test_models", []):
        body = {"model": model, "messages": [{"role": "user", "content": test_cfg["test_message"]}], "max_tokens": test_cfg["max_tokens"], "stream": False}
        status, data, err, elapsed = api_request("POST", f"{base_url}/chat/completions", headers, body, test_cfg["timeout_seconds"])
        r = {"model": model, "ok": 200 <= status < 300, "status_code": status, "elapsed_ms": elapsed, "error": err}
        if r["ok"]:
            try:
                r["response_text"] = data["choices"][0]["message"]["content"]
                r["usage"] = data.get("usage", {})
                r["model_used"] = data.get("model", model)
            except (KeyError, IndexError):
                r["response_text"] = str(data)[:200]
        else:
            r["error_detail"] = str(data)[:300]
        result["models_test"][model] = r
        time.sleep(0.5)

    return result

def main():
    config = load_config()
    test_cfg = config.get("test_config", {})

    print("=" * 70)
    print("  Deep Student — All Vendor Connectivity Test")
    print(f"  Started: {datetime.now(timezone.utc).isoformat()}")
    print("=" * 70)

    results = {"started": datetime.now(timezone.utc).isoformat(), "vendors": {}}
    total_ok = 0
    total_fail = 0
    total_skip = 0

    for vendor_key, vendor_cfg in config.get("vendors", {}).items():
        print(f"\n--- {vendor_cfg['name']} ({vendor_key}) ---")
        r = test_vendor(vendor_key, vendor_cfg, test_cfg)
        results["vendors"][vendor_key] = r

        print(f"  Status: {r['status']}")
        if r["status"] == "skipped":
            total_skip += 1
            print(f"  Reason: {r['reason']}")
            continue

        me = r.get("models_endpoint", {})
        print(f"  Models endpoint: {'OK' if me.get('ok') else 'FAIL'} ({me.get('model_count', 0)} models, {me.get('elapsed_ms', 0)}ms)")

        for model, mt in r.get("models_test", {}).items():
            ok_str = "PASS" if mt["ok"] else "FAIL"
            resp = mt.get("response_text", mt.get("error_detail", ""))[:80]
            print(f"  [{ok_str}] {model}: {resp} ({mt['elapsed_ms']}ms)")
            if mt["ok"]: total_ok += 1
            else: total_fail += 1

    results["summary"] = {"total_vendors": len(config.get("vendors", {})), "skipped": total_skip, "tested": total_ok + total_fail, "passed": total_ok, "failed": total_fail, "finished": datetime.now(timezone.utc).isoformat()}

    # Save results
    out_path = test_cfg.get("save_results_to", "connectivity_test_results.json")
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2, ensure_ascii=False, default=str)

    s = results["summary"]
    print(f"\n{'=' * 70}")
    print(f"  SUMMARY: {s['tested']} models tested across {s['total_vendors']} vendors")
    print(f"  Passed: {s['passed']} | Failed: {s['failed']} | Skipped: {s['skipped']}")
    print(f"  Results saved to: {out_path}")
    print(f"{'=' * 70}")

    return 0 if s["failed"] == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
