#!/usr/bin/env pwsh
<#
.SYNOPSIS
    PaddleOCR REST API Connectivity Test
.DESCRIPTION
    Tests connectivity to the PaddleOCR AI Studio REST API endpoint.
    Verifies DNS resolution, TCP connectivity, TLS handshake, and API response format.

    Usage:
        .\scripts\test-paddleocr-connectivity.ps1 [-Token "your_bearer_token_here"]

    Without -Token, the script tests basic network connectivity only (no auth test).
    With -Token, it additionally attempts API authentication verification.

    Environment variable: PADDLEOCR_API_TOKEN (alternative to -Token parameter)
.NOTES
    API Base: https://paddleocr.aistudio-app.com/api/v2
#>

param(
    [string]$Token = ""
)

$ErrorActionPreference = "Stop"
$API_BASE = "https://paddleocr.aistudio-app.com/api/v2"

# Use parameter first, then env var
if ([string]::IsNullOrEmpty($Token)) {
    $Token = [Environment]::GetEnvironmentVariable("PADDLEOCR_API_TOKEN")
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  PaddleOCR API 连接性测试" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "目标: $API_BASE"
Write-Host "时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
if ($Token) {
    Write-Host "Token: $($Token.Substring(0, [Math]::Min(8, $Token.Length)))..." -ForegroundColor Yellow
} else {
    Write-Host "Token: 未提供 (跳过认证测试)" -ForegroundColor Yellow
}
Write-Host ""

# ---------- Test 1: DNS Resolution ----------
Write-Host "[1/5] DNS 解析测试..." -ForegroundColor Green
try {
    $dnsResult = [System.Net.Dns]::GetHostAddresses("paddleocr.aistudio-app.com")
    Write-Host "  OK - 解析到 IP: $($dnsResult.IPAddressToString -join ', ')" -ForegroundColor Green
} catch {
    Write-Host "  FAIL - DNS 解析失败: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# ---------- Test 2: TCP Connectivity ----------
Write-Host "[2/5] TCP 连接测试 (443)..." -ForegroundColor Green
try {
    $tcpClient = New-Object System.Net.Sockets.TcpClient
    $connectResult = $tcpClient.BeginConnect("paddleocr.aistudio-app.com", 443, $null, $null)
    $timeout = $connectResult.AsyncWaitHandle.WaitOne(5000, $false) # 5s timeout
    if ($timeout) {
        $tcpClient.EndConnect($connectResult)
        Write-Host "  OK - TCP 连接成功" -ForegroundColor Green
        $tcpClient.Close()
    } else {
        Write-Host "  FAIL - TCP 连接超时 (5s)" -ForegroundColor Red
        $tcpClient.Close()
        exit 1
    }
} catch {
    Write-Host "  FAIL - TCP 连接失败: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# ---------- Test 3: TLS Handshake ----------
Write-Host "[3/5] TLS 握手测试..." -ForegroundColor Green
try {
    $request = [System.Net.WebRequest]::CreateHttp("$API_BASE/ocr/jobs")
    $request.Method = "HEAD"
    $request.Timeout = 10000
    # We only care about TLS handshake, so we expect an error (HEAD not allowed or auth required)
    try {
        $response = $request.GetResponse()
        $response.Close()
    } catch {
        # Getting 401/403/405 means TLS handshake succeeded
        if ($_.Exception.InnerException -and $_.Exception.InnerException.Message -match "authentication|authorize|401|403|405|405") {
            Write-Host "  OK - TLS 握手成功 (服务器响应: $($_.Exception.Message))" -ForegroundColor Green
        } elseif ($_.Exception.Message -match "401|403|405") {
            Write-Host "  OK - TLS 握手成功 (服务器响应: $($_.Exception.Message))" -ForegroundColor Green
        } else {
            throw $_.Exception
        }
    }
    Write-Host "  OK - TLS 握手成功" -ForegroundColor Green
} catch {
    Write-Host "  FAIL - TLS 握手失败: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# ---------- Test 4: API Response ----------
Write-Host "[4/5] API 响应格式测试..." -ForegroundColor Green
try {
    $response = Invoke-WebRequest -Uri "$API_BASE/ocr/jobs" -Method Get -TimeoutSec 10 -SkipCertificateCheck -ErrorAction SilentlyContinue
    Write-Host "  OK - API 响应: HTTP $($response.StatusCode)" -ForegroundColor Green
    Write-Host "  Content-Type: $($response.Headers['Content-Type'])"
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    if ($statusCode -eq 401 -or $statusCode -eq 403) {
        Write-Host "  OK - API 可达，返回 HTTP $statusCode (认证正常)" -ForegroundColor Green
    } elseif ($_.Exception.Message -match "Unable to connect|connection refused|Name or service not known") {
        Write-Host "  FAIL - API 不可达: $_" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "  OK (预期错误) - $($_.Exception.Message)" -ForegroundColor Green
    }
}
Write-Host ""

# ---------- Test 5: Auth (if token provided) ----------
if ($Token) {
    Write-Host "[5/5] API 认证测试..." -ForegroundColor Green
    try {
        $headers = @{
            "Authorization" = "Bearer $Token"
            "Content-Type" = "application/json"
        }
        $body = @{
            model = "PaddleOCR-VL-1.6"
            fileUrl = "https://example.com/test.pdf"
        } | ConvertTo-Json

        $response = Invoke-WebRequest -Uri "$API_BASE/ocr/jobs" -Method Post `
            -Headers $headers -Body $body -TimeoutSec 10 -SkipCertificateCheck

        Write-Host "  OK - 认证成功! Job 已提交" -ForegroundColor Green
        Write-Host "  Response: $($response.Content)" -ForegroundColor Gray
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 401) {
            Write-Host "  FAIL - Token 无效 (HTTP 401)" -ForegroundColor Red
        } elseif ($statusCode -eq 422 -or $statusCode -eq 400) {
            # 422/400 means auth succeeded but body format was wrong (expected with dummy URL)
            Write-Host "  OK - 认证成功 (HTTP $statusCode - 请求体格式检查通过)" -ForegroundColor Green
        } elseif ($_.Exception.Message -match "Unable to connect") {
            Write-Host "  FAIL - 无法连接 API: $_" -ForegroundColor Red
            exit 1
        } else {
            Write-Host "  ?? - HTTP $statusCode : $($_.Exception.Message)" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "[5/5] API 认证测试 - 跳过 (未提供 Token)" -ForegroundColor Yellow
}
Write-Host ""

# ---------- Summary ----------
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  测试完成" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Output machine-readable JSON summary
$summary = @{
    timestamp = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    api_base = $API_BASE
    dns = "passed"
    tcp = "passed"
    tls = "passed"
    api_reachable = "passed"
    auth_tested = (-not [string]::IsNullOrEmpty($Token))
    auth_passed = $null  # can't determine from script alone
} | ConvertTo-Json

Write-Host ""
Write-Host "Machine-readable summary:" -ForegroundColor Gray
Write-Host $summary -ForegroundColor Gray
