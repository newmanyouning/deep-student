#!/usr/bin/env python3
"""
Model Connectivity & Registry Test Script

Reads test_api_keys.json, then for each configured platform:
  1. Lists available models via GET /models
  2. Sends a minimal chat completion request for each test model
  3. Compares the API model list against the builtin model definitions in Rust
  4. Outputs a JSON report with all findings

Usage:
  python scripts/test_model_connectivity.py [--config scripts/test_api_keys.json]

Environment variables (override JSON keys):
  TEST_ALIYUN_DASHSCOPE_KEY
  TEST_SILICONFLOW_KEY
"""

import json
import os
import sys
import time
import urllib.request
import urllib.error
import ssl
from dataclasses import dataclass, field, asdict
from typing import Any, Optional

# ---------------------------------------------------------------------------
# Builtin model reference lists (extracted from Rust source)
# These are the authoritative models registered in builtin_vendors.rs
# ---------------------------------------------------------------------------

# From builtin_vendors.rs: vendor.id = "builtin-qwen" (provider_type = "qwen")
BUILTIN_ALIYUN_DASHSCOPE_MODELS: list[str] = [
    "qwen3-max",
    "qwen3.5-plus",
    "qwen3.5-flash",
    "qwen-plus",
    "qwq-plus",
    "qwen3.5-397b-a17b",
    "qwen3.5-122b-a10b",
]

# From builtin_vendors.rs: vendor.id = "builtin-siliconflow" (provider_type = "siliconflow")
# And from siliconflow.rs: BuiltinModelConfig entries (env-var-backed)
BUILTIN_SILICONFLOW_MODELS: list[str] = [
    "Qwen/Qwen3-8B",
    "zai-org/GLM-4.6V",
    "BAAI/bge-m3",
]

# ---------------------------------------------------------------------------
# Data structures
# ---------------------------------------------------------------------------


@dataclass
class ApiModelEntry:
    """A single model entry returned by the /models endpoint."""
    id: str
    object_type: str = ""
    owned_by: str = ""

    @staticmethod
    def from_json(obj: dict) -> "ApiModelEntry":
        return ApiModelEntry(
            id=obj.get("id", obj.get("model", "")),
            object_type=obj.get("object", ""),
            owned_by=obj.get("owned_by", ""),
        )


@dataclass
class ChatCompletionResult:
    model: str
    success: bool
    status_code: int
    elapsed_ms: int
    response_body: Optional[dict] = None
    error_message: str = ""
    # extracted fields
    response_text: str = ""
    usage_input_tokens: int = 0
    usage_output_tokens: int = 0


@dataclass
class PlatformTestResult:
    platform: str
    base_url: str
    api_key_configured: bool
    api_key_placeholder: bool
    models_endpoint_ok: bool
    models_endpoint_error: str = ""
    api_models: list[ApiModelEntry] = field(default_factory=list)
    api_model_ids: list[str] = field(default_factory=list)
    chat_tests: list[ChatCompletionResult] = field(default_factory=list)
    builtin_models: list[str] = field(default_factory=list)
    models_missing_from_api: list[str] = field(default_factory=list)
    models_not_in_builtin: list[str] = field(default_factory=list)


@dataclass
class TestReport:
    timestamp: str
    config_file: str
    results: dict[str, Any]  # platform_name -> PlatformTestResult dict
    summary: dict[str, Any]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

PLACEHOLDER_PREFIX = "PLACEHOLDER_"


def _is_placeholder(key: str) -> bool:
    return key.upper().startswith(PLACEHOLDER_PREFIX) or key.strip() == ""


def _utc_now_iso() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def _make_request(
    method: str,
    url: str,
    headers: dict[str, str],
    body: Optional[bytes] = None,
    timeout: int = 15,
) -> tuple[int, dict[str, Any], str]:
    """Make an HTTP request and return (status_code, parsed_json, error_string)."""
    ctx = ssl.create_default_context()
    req = urllib.request.Request(url, data=body, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=timeout, context=ctx) as resp:
            raw = resp.read()
            status = resp.status
            try:
                data = json.loads(raw)
            except json.JSONDecodeError:
                data = {"raw": raw.decode("utf-8", errors="replace")}
            return status, data, ""
    except urllib.error.HTTPError as e:
        status = e.code
        try:
            body_raw = e.read()
            data = json.loads(body_raw)
        except Exception:
            data = {"error": str(e)}
        return status, data, str(e)
    except urllib.error.URLError as e:
        return 0, {"error": str(e.reason)}, str(e)
    except Exception as e:
        return 0, {"error": str(e)}, str(e)


def fetch_models(
    base_url: str, api_key: str, timeout: int = 15
) -> tuple[bool, list[ApiModelEntry], str]:
    """Call GET /models and return (ok, models, error)."""
    url = f"{base_url.rstrip('/')}/models"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }
    status, data, err = _make_request("GET", url, headers, timeout=timeout)

    if status == 0 or status >= 400:
        return False, [], err or data.get("error", {}).get("message", str(data))

    # The /models endpoint typically returns {"object": "list", "data": [...]}
    raw_models = []
    if isinstance(data, dict):
        raw_models = data.get("data", [])
    if not raw_models and isinstance(data, list):
        raw_models = data

    models = [ApiModelEntry.from_json(m) for m in raw_models if isinstance(m, dict)]
    return True, models, ""


def test_chat_completion(
    base_url: str,
    api_key: str,
    model: str,
    timeout: int = 15,
) -> ChatCompletionResult:
    """Send a minimal chat completion request and return the result."""
    url = f"{base_url.rstrip('/')}/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }
    body = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": "Hi"}],
        "max_tokens": 5,
        "stream": False,
    }).encode("utf-8")

    start = time.monotonic()
    status, data, err = _make_request("POST", url, headers, body, timeout=timeout)
    elapsed = int((time.monotonic() - start) * 1000)

    result = ChatCompletionResult(
        model=model,
        success=status == 0 or 200 <= status < 300,
        status_code=status,
        elapsed_ms=elapsed,
        error_message=err,
    )

    if result.success and isinstance(data, dict):
        result.response_body = data
        # Extract response text
        try:
            result.response_text = data["choices"][0]["message"]["content"]
        except (KeyError, IndexError, TypeError):
            result.response_text = "<could not extract>"
        # Extract usage
        usage = data.get("usage", {})
        result.usage_input_tokens = usage.get("prompt_tokens", 0) or usage.get("input_tokens", 0)
        result.usage_output_tokens = usage.get("completion_tokens", 0) or usage.get("output_tokens", 0)
    else:
        if isinstance(data, dict):
            msg = data.get("error", {}).get("message", json.dumps(data, ensure_ascii=False))
            result.error_message = result.error_message or msg

    return result


def diff_models(
    api_model_ids: list[str],
    builtin_ids: list[str],
) -> tuple[list[str], list[str]]:
    """Return (missing_from_api, not_in_builtin)."""
    api_set = set(api_model_ids)
    builtin_set = set(builtin_ids)
    missing = sorted(builtin_set - api_set)
    extra = sorted(api_set - builtin_set)
    return missing, extra


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def load_config(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def resolve_api_key(platform_key: str, env_var_name: str, cfg_val: str) -> str:
    """Resolve API key: env var overrides JSON, placeholder means skip."""
    env_val = os.environ.get(env_var_name, "")
    if env_val:
        return env_val
    return cfg_val


def run_platform_test(
    platform_name: str,
    platform_cfg: dict,
    builtin_models: list[str],
) -> PlatformTestResult:
    """Run all tests for a single platform."""
    base_url = platform_cfg.get("base_url", "")
    raw_key = platform_cfg.get("api_key", "")
    test_models = platform_cfg.get("test_models", [])

    env_var = f"TEST_{platform_name.upper()}_KEY"
    api_key = resolve_api_key(platform_name, env_var, raw_key)

    result = PlatformTestResult(
        platform=platform_name,
        base_url=base_url,
        api_key_configured=bool(api_key) and not _is_placeholder(api_key),
        api_key_placeholder=_is_placeholder(api_key),
        models_endpoint_ok=False,
        builtin_models=builtin_models,
    )

    if _is_placeholder(api_key) or not api_key:
        # Cannot run tests without a real key
        return result

    # 1. Fetch models from API
    ok, models, err = fetch_models(base_url, api_key)
    result.models_endpoint_ok = ok
    if not ok:
        result.models_endpoint_error = err
    else:
        result.api_models = models
        result.api_model_ids = [m.id for m in models]

    # 2. Diff models
    missing, extra = diff_models(result.api_model_ids, builtin_models)
    result.models_missing_from_api = missing
    result.models_not_in_builtin = extra

    # 3. Test chat completion for each test model
    for model_name in test_models:
        chat_result = test_chat_completion(base_url, api_key, model_name)
        result.chat_tests.append(chat_result)

    return result


def build_report(
    config_path: str,
    results: dict[str, PlatformTestResult],
) -> TestReport:
    """Build the final test report with summary."""
    total_platforms = len(results)
    tested_platforms = sum(1 for r in results.values() if r.api_key_configured)
    skipped_platforms = total_platforms - tested_platforms

    total_chat_tests = 0
    passed_chat_tests = 0
    failed_chat_tests = 0
    models_endpoint_ok = 0

    for r in results.values():
        total_chat_tests += len(r.chat_tests)
        passed_chat_tests += sum(1 for c in r.chat_tests if c.success)
        failed_chat_tests += sum(1 for c in r.chat_tests if not c.success)
        if r.models_endpoint_ok:
            models_endpoint_ok += 1

    summary = {
        "total_platforms": total_platforms,
        "tested_platforms": tested_platforms,
        "skipped_platforms": skipped_platforms,
        "models_endpoint_ok": models_endpoint_ok,
        "total_chat_tests": total_chat_tests,
        "passed_chat_tests": passed_chat_tests,
        "failed_chat_tests": failed_chat_tests,
        "all_passed": failed_chat_tests == 0,
    }

    serializable_results: dict[str, Any] = {}
    for name, r in results.items():
        serializable_results[name] = asdict(r)

    return TestReport(
        timestamp=_utc_now_iso(),
        config_file=config_path,
        results=serializable_results,
        summary=summary,
    )


def main():
    config_path = "scripts/test_api_keys.json"
    if len(sys.argv) > 1:
        config_path = sys.argv[1]

    if not os.path.isfile(config_path):
        print(json.dumps({
            "error": f"Config file not found: {config_path}",
            "hint": "Create the file or pass a different path as argument.",
            "expected_format": {
                "aliyun_dashscope": {
                    "api_key": "sk-...",
                    "base_url": "https://dashscope.aliyuncs.com/compatible-mode/v1",
                    "test_models": ["qwen-plus", "qwen-turbo", "qwq-plus"],
                },
                "siliconflow": {
                    "api_key": "sk-...",
                    "base_url": "https://api.siliconflow.cn/v1",
                    "test_models": ["Qwen/Qwen3-8B", "zai-org/GLM-4.6V", "BAAI/bge-m3"],
                },
            },
        }, indent=2, ensure_ascii=False))
        sys.exit(1)

    config = load_config(config_path)

    platform_map = {
        "aliyun_dashscope": BUILTIN_ALIYUN_DASHSCOPE_MODELS,
        "siliconflow": BUILTIN_SILICONFLOW_MODELS,
    }

    results: dict[str, PlatformTestResult] = {}

    for platform_name, builtin_list in platform_map.items():
        if platform_name not in config:
            # Platform not in config, create a skipped result
            results[platform_name] = PlatformTestResult(
                platform=platform_name,
                base_url="",
                api_key_configured=False,
                api_key_placeholder=True,
                models_endpoint_ok=False,
                models_endpoint_error="platform not in config file",
                builtin_models=builtin_list,
            )
            continue

        platform_cfg = config[platform_name]
        result = run_platform_test(platform_name, platform_cfg, builtin_list)
        results[platform_name] = result

    report = build_report(config_path, results)

    # Output pure JSON to stdout
    print(json.dumps(asdict(report), indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
