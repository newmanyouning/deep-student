#!/bin/zsh
# ============================================================
# Deep Student iPad 版每周续签脚本
# 用法:  ~/deep-student/renew-ipad.sh
# 前提:  ① iPad 用数据线连着 Mac 并已解锁信任
#        ② Mac 能上网（续签时要连 Apple 服务器验证）
#        ③ Xcode 里已登录 Apple ID（Settings → Accounts）
# 耗时:  约 3-6 分钟
# ============================================================
set -o pipefail
cd /Users/mac/deep-student

export PATH="/opt/homebrew/bin:$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
unset CI

DEVICE_ID="00008112-0015295C0A99A01E"
APP_PATH="$HOME/Library/Developer/Xcode/DerivedData/deep-student-cwjhdczzrfkpzseprfgjvtkrxqmf/Build/Products/debug-iphoneos/Deep Student.app"
ADDR_FILE="$TMPDIR/com.lanxia.deepstudent-server-addr"

echo "[1/4] 启动 Tauri 配置服务（约 2-3 分钟，请等待）..."
nohup npx tauri ios build --target aarch64 --open >/tmp/tauri-open.log 2>&1 &
SERVER_PID=$!

# 轮询等待服务端口就绪（前端构建完成后才会监听）
PORT=""
for i in {1..60}; do
  sleep 5
  PORT=$(cat "$ADDR_FILE" 2>/dev/null | cut -d: -f2)
  [ -n "$PORT" ] && lsof -nP -iTCP:$PORT -sTCP:LISTEN >/dev/null 2>&1 && break
  PORT=""
done
if [ -z "$PORT" ]; then
  echo "❌ 配置服务启动失败，日志: /tmp/tauri-open.log"
  kill $SERVER_PID 2>/dev/null
  exit 1
fi
echo "      服务就绪: 127.0.0.1:$PORT"

echo "[2/4] 续签签名 + 编译（Rust 已缓存，这步很快）..."
xcodebuild -project src-tauri/gen/apple/deep-student.xcodeproj \
  -scheme deep-student_iOS \
  -configuration Debug \
  -destination "generic/platform=iOS" \
  -allowProvisioningUpdates build 2>&1 | tail -3
if [ $? -ne 0 ]; then
  echo "❌ 构建失败。常见原因："
  echo "   · Xcode 账号登录过期 → 打开 Xcode → Settings → Accounts 重新登录"
  echo "   · iPad 需要在设置里重新信任开发者证书"
  kill $SERVER_PID 2>/dev/null
  exit 1
fi

echo "[3/4] 安装到 iPad..."
xcrun devicectl device install app --device $DEVICE_ID "$APP_PATH"
if [ $? -ne 0 ]; then
  echo "❌ 安装失败。检查：iPad 是否已连接并解锁；数据线是否正常"
  kill $SERVER_PID 2>/dev/null
  exit 1
fi

kill $SERVER_PID 2>/dev/null
# 清掉被 --open 顺带弹出的 Xcode 工程窗口干扰（可选，保持 Xcode 主进程）
echo "[4/4] ✅ 续签完成！到 iPad 上打开 Deep Student 即可"
echo "      首次打开若提示验证，稍等几秒联网自动通过；若要求信任：设置→通用→VPN与设备管理→信任"
