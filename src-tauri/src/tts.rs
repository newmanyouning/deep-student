/// Tauri TTS 模块 - 可选的系统级语音合成
///
/// 当 WebView 的 Web Speech API 不可用时，使用系统 TTS 作为备选方案
///
/// 平台支持：
/// - Windows: PowerShell + SAPI COM 子进程（零新增依赖，与 macOS/Linux 的 shell 方式一致）
/// - macOS: say 命令（同步阻塞，等待朗读完成）
/// - Linux: espeak 命令（同步阻塞，等待朗读完成）
use crate::models::AppError;

#[cfg(target_os = "windows")]
use std::process::Child;
#[cfg(target_os = "windows")]
use std::sync::Mutex;

/// TTS 请求参数
#[derive(Debug, serde::Deserialize)]
pub struct TTSRequest {
    pub text: String,
    pub lang: Option<String>,
    pub rate: Option<f32>,
    pub volume: Option<f32>,
}

/// 检查 TTS 是否可用
#[tauri::command]
pub async fn tts_check_available() -> Result<bool, AppError> {
    #[cfg(target_os = "windows")]
    {
        // Windows: 真实探测 PowerShell + SAPI COM 是否可用（不再恒 true 谎报）
        // 每次调用会启动一次 PowerShell 探测（约几百毫秒），
        // 该命令仅在 Web Speech API 回退时被调用，频率低，性能可接受
        Ok(check_windows_available())
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 通常都有 TTS
        Ok(true)
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 需要检查 espeak 或 speech-dispatcher
        use std::process::Command;
        let has_espeak = Command::new("which")
            .arg("espeak")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        Ok(has_espeak)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Ok(false)
    }
}

/// 朗读文本
#[tauri::command]
pub async fn tts_speak(
    text: String,
    lang: Option<String>,
    rate: Option<f32>,
    volume: Option<f32>,
) -> Result<(), AppError> {
    println!(
        "🔊 TTS 朗读: lang={:?}, rate={:?}, volume={:?}",
        lang, rate, volume
    );

    #[cfg(target_os = "windows")]
    {
        speak_windows(&text, lang.as_deref(), rate, volume).await
    }

    #[cfg(target_os = "macos")]
    {
        speak_macos(&text, lang.as_deref(), rate, volume).await
    }

    #[cfg(target_os = "linux")]
    {
        speak_linux(&text, lang.as_deref(), rate, volume).await
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(AppError::not_implemented("当前平台不支持 TTS"))
    }
}

/// 停止朗读
#[tauri::command]
pub async fn tts_stop() -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        stop_windows()
    }

    #[cfg(not(target_os = "windows"))]
    {
        // macOS/Linux 朗读为同步阻塞调用（等待朗读完成），无需主动停止；
        // 如需支持停止，需将 speak_macos/speak_linux 改为 spawn + 后台等待模式（同 Windows）
        println!("🛑 停止 TTS 朗读（当前平台朗读为同步调用，无需主动停止）");
        Ok(())
    }
}

// ============================================================================
// Windows 实现：PowerShell + SAPI COM 子进程
//
// 设计说明：
// - 零新增 crate：复用系统自带 PowerShell 的 SAPI COM（与 macOS say / Linux espeak
//   的 shell 子进程方式一致）
// - 文本安全：文本 base64 编码后嵌入脚本，脚本内解码，避免引号/特殊字符/换行转义问题
// - 停止机制：子进程存入全局槽位 CURRENT_TTS，tts_stop 直接 kill（SAPI 在
//   powershell.exe 进程内发声，杀进程即停声）；后台线程轮询 try_wait 回收
// ============================================================================

/// 朗读文本的最大字符数（防御性截断）
/// Windows 命令行上限 32767 字符；中文 UTF-8 3 字节/字，base64 后约 4 字符/字
#[cfg(target_os = "windows")]
const MAX_TEXT_CHARS: usize = 2000;

/// 当前正在朗读的 TTS 子进程（仅 Windows 使用，供 tts_stop 真正停止）
/// 子进程持有权始终在槽位内：朗读线程轮询 try_wait，停止线程 kill + wait
#[cfg(target_os = "windows")]
static CURRENT_TTS: Mutex<Option<RunningTts>> = Mutex::new(None);

/// 正在进行的 Windows TTS 子进程
#[cfg(target_os = "windows")]
struct RunningTts {
    /// 子进程 PID：朗读线程据此识别"槽位中的进程是否还是我负责的那个"
    pid: u32,
    /// PowerShell 子进程（SAPI COM 在其进程内发声，杀进程即停声）
    child: Child,
}

/// 语速映射：TTSRequest.rate 为倍率（1.0 正常，0.5 半速 / 2.0 两倍速）
/// SAPI Rate 范围 -10..10（0 为正常），线性映射后夹紧
/// 与 macOS/Linux 的语义对齐（那里按 175wpm * rate 换算）
#[cfg(target_os = "windows")]
fn map_rate(rate: f32) -> i32 {
    (((rate - 1.0) * 10.0).round() as i32).clamp(-10, 10)
}

/// 音量映射：TTSRequest.volume 为 0.0..1.0（默认 1.0 = 100%）
/// SAPI Volume 范围 0..100（默认 100），线性映射后夹紧
#[cfg(target_os = "windows")]
fn map_volume(volume: f32) -> i32 {
    ((volume.clamp(0.0, 1.0)) * 100.0).round() as i32
}

/// 构建 PowerShell + SAPI COM 朗读脚本
/// - 文本经 base64 编码嵌入，脚本内 [Text.Encoding]::UTF8 解码，彻底规避引号/换行转义
/// - 脚本本身保持纯 ASCII，规避 PowerShell 5.1 命令行的代码页/编码问题
/// - Speak 为同步调用，进程内发声；[void] 抑制返回值输出
#[cfg(target_os = "windows")]
fn build_sapi_script(text: &str, rate: Option<f32>, volume: Option<f32>) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    // UTF-8 base64 编码（中文等多字节字符安全）
    let encoded = STANDARD.encode(text.as_bytes());

    let mut script = String::from("$v = New-Object -ComObject SAPI.SpVoice; ");
    if let Some(r) = rate {
        script.push_str(&format!("$v.Rate = {}; ", map_rate(r)));
    }
    if let Some(v) = volume {
        script.push_str(&format!("$v.Volume = {}; ", map_volume(v)));
    }
    script.push_str(&format!(
        "$text = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{}')); ",
        encoded
    ));
    script.push_str("[void]$v.Speak($text);");
    script
}

/// 获取全局子进程槽位锁（毒锁兜底：单次 panic 不阻塞后续调用）
#[cfg(target_os = "windows")]
fn lock_current() -> std::sync::MutexGuard<'static, Option<RunningTts>> {
    CURRENT_TTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 登记新子进程：覆盖旧朗读（先终止旧进程，避免声音叠加），并启动后台等待线程
#[cfg(target_os = "windows")]
fn register_current(child: Child) {
    let pid = child.id();
    let old = {
        let mut guard = lock_current();
        // MutexGuard deref 到 Option<RunningTts>, replace 直接收 RunningTts
        guard.replace(RunningTts { pid, child })
    };
    // 在锁外终止旧进程，避免持锁等待
    if let Some(mut old) = old {
        // 旧进程可能已自然结束：kill/wait 失败均可忽略，wait 不会 panic
        let _ = old.child.kill();
        let _ = old.child.wait();
    }
    // 后台线程轮询等待子进程结束，完成后自动清理槽位
    std::thread::spawn(move || wait_for_child(pid));
}

/// 后台等待线程：轮询 try_wait，子进程结束（自然结束或被 kill）后清空槽位
/// 只清理"自己负责的 PID"，防止旧线程误清新朗读
#[cfg(target_os = "windows")]
fn wait_for_child(pid: u32) {
    loop {
        {
            let mut guard = lock_current();
            match guard.as_mut() {
                // 槽位已清空（tts_stop 已接管）或已被新朗读替换 → 本线程退出
                None => return,
                Some(running) if running.pid != pid => return,
                Some(running) => match running.child.try_wait() {
                    // 自然结束：读取 stderr 便于诊断，然后清空槽位
                    Ok(Some(status)) => {
                        let stderr = running.child.stderr.take();
                        *guard = None;
                        drop(guard);
                        if !status.success() {
                            log_child_stderr(stderr);
                        }
                        return;
                    }
                    // try_wait 失败：按已结束处理，避免死循环
                    Err(_) => {
                        *guard = None;
                        return;
                    }
                    // 仍在朗读：释放锁，稍等再查
                    Ok(None) => {}
                },
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// 输出子进程 stderr（便于排查 SAPI 脚本失败原因）
#[cfg(target_os = "windows")]
fn log_child_stderr(stderr: Option<std::process::ChildStderr>) {
    use std::io::Read;
    if let Some(mut err) = stderr {
        let mut buf = String::new();
        if err.read_to_string(&mut buf).is_ok() {
            let msg = buf.trim();
            if !msg.is_empty() {
                println!("⚠️ Windows TTS 子进程输出: {}", msg);
            }
        }
    }
}

/// 探测 Windows TTS 是否可用：能否启动 PowerShell 并创建 SAPI.SpVoice COM 对象
/// 返回 false 的场景：无 PowerShell、无 SAPI 语音引擎、COM 注册异常等
#[cfg(target_os = "windows")]
fn check_windows_available() -> bool {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // 探测脚本：创建 COM 对象失败时 PowerShell 退出码非 0
    let probe = "$null = New-Object -ComObject SAPI.SpVoice";
    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", probe])
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW：不弹出控制台窗口
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Windows 停止朗读：kill 当前 PowerShell 子进程（SAPI 在其进程内发声，杀进程即停声）
#[cfg(target_os = "windows")]
fn stop_windows() -> Result<(), AppError> {
    let taken = lock_current().take();
    match taken {
        Some(mut running) => {
            let pid = running.pid;
            // 进程可能已自然结束：kill/wait 失败均忽略，wait 不会 panic
            let _ = running.child.kill();
            let _ = running.child.wait();
            println!("🛑 已停止 TTS 朗读（PID={}）", pid);
        }
        None => {
            println!("🛑 停止 TTS 朗读（当前无进行中的朗读）");
        }
    }
    Ok(())
}

/// Windows 朗读：PowerShell + SAPI COM（同步 Speak，进程内发声）
/// - 返回时机：spawn 成功后立即返回 Ok，后台线程负责等待完成
/// - 停止：tts_stop 直接 kill 子进程
#[cfg(target_os = "windows")]
async fn speak_windows(
    text: &str,
    _lang: Option<&str>,
    rate: Option<f32>,
    volume: Option<f32>,
) -> Result<(), AppError> {
    use std::os::windows::process::CommandExt;
    use std::process::{Command, Stdio};

    // 空文本直接成功返回（与 Web Speech API 的空朗读行为一致）
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }

    // 防御性截断：避免 PowerShell 命令行超长（见 MAX_TEXT_CHARS）
    let text = if text.chars().count() > MAX_TEXT_CHARS {
        println!(
            "⚠️ TTS 文本过长（{} 字符），已截断至 {} 字符",
            text.chars().count(),
            MAX_TEXT_CHARS
        );
        text.chars().take(MAX_TEXT_CHARS).collect::<String>()
    } else {
        text.to_string()
    };

    // 说明：lang 暂不映射到 SAPI 语音 —— SAPI 默认使用系统语音；
    // 按语言选择语音需匹配 GetVoices() 描述（描述随系统区域本地化），跨区域不稳定，后续可增强
    let script = build_sapi_script(&text, rate, volume);

    let child = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script.as_str()])
        // CREATE_NO_WINDOW (0x08000000)：不弹出控制台窗口
        .creation_flags(0x0800_0000)
        // 捕获 stderr 便于诊断（后台线程读取）
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::internal(format!("启动 PowerShell TTS 失败: {}", e)))?;

    // 登记子进程（覆盖旧朗读），后台线程等待完成
    register_current(child);

    Ok(())
}

// ============================================================================
// macOS / Linux 实现（保持原有行为，未改动）
// ============================================================================

#[cfg(target_os = "macos")]
async fn speak_macos(
    text: &str,
    lang: Option<&str>,
    rate: Option<f32>,
    _volume: Option<f32>,
) -> Result<(), AppError> {
    use std::process::Command;

    // 使用 macOS 的 say 命令
    let mut cmd = Command::new("say");

    // 设置语言/语音
    if let Some(lang_code) = lang {
        let voice = match lang_code {
            "zh-CN" | "zh" => "Ting-Ting",
            "en-US" | "en" => "Alex",
            "ja-JP" | "ja" => "Kyoko",
            _ => "Alex",
        };
        cmd.arg("-v").arg(voice);
    }

    // 设置语速（say 命令使用 words per minute）
    if let Some(r) = rate {
        let wpm = (175.0 * r) as u32; // 默认 175 wpm
        cmd.arg("-r").arg(wpm.to_string());
    }

    cmd.arg(text);

    let output = cmd
        .output()
        .map_err(|e| AppError::internal(format!("执行 say 命令失败: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::internal(format!("TTS 失败: {}", stderr)))
    }
}

#[cfg(target_os = "linux")]
async fn speak_linux(
    text: &str,
    lang: Option<&str>,
    rate: Option<f32>,
    _volume: Option<f32>,
) -> Result<(), AppError> {
    use std::process::Command;

    // 使用 espeak 命令
    let mut cmd = Command::new("espeak");

    // 设置语言
    if let Some(lang_code) = lang {
        let espeak_lang = match lang_code {
            "zh-CN" | "zh" => "zh",
            "en-US" | "en" => "en",
            "ja-JP" | "ja" => "ja",
            _ => "en",
        };
        cmd.arg("-v").arg(espeak_lang);
    }

    // 设置语速 (espeak 使用 words per minute)
    if let Some(r) = rate {
        let wpm = (175.0 * r) as u32;
        cmd.arg("-s").arg(wpm.to_string());
    }

    cmd.arg(text);

    let output = cmd
        .output()
        .map_err(|e| AppError::internal(format!("执行 espeak 命令失败: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(AppError::internal(format!("TTS 失败: {}", stderr)))
    }
}

// ============================================================================
// 单元测试（Windows 专用：只测纯函数，不真正发声）
// ============================================================================

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    /// 从脚本中提取 base64 文本并解码（验证文本完整往返）
    fn decode_text_from_script(script: &str) -> String {
        const MARKER: &str = "FromBase64String('";
        let start = script.find(MARKER).unwrap() + MARKER.len();
        let end = script[start..].find('\'').unwrap() + start;
        let b64 = &script[start..end];
        String::from_utf8(STANDARD.decode(b64).unwrap()).unwrap()
    }

    #[test]
    fn test_script_never_leaks_raw_text() {
        // 脚本中不允许出现原文（只允许 base64），否则存在引号/换行转义风险
        let text = "Hello 世界！<tag> \"双引号\" '单引号' \n 换行 & $var";
        let script = build_sapi_script(text, None, None);
        assert!(!script.contains(text));
        // 且 base64 解码后可还原原文
        assert_eq!(decode_text_from_script(&script), text);
    }

    #[test]
    fn test_base64_roundtrip_long_chinese() {
        // 长中文文本（多字节 UTF-8）完整往返
        let text = "深度学习与人工智能技术".repeat(200);
        let script = build_sapi_script(&text, Some(1.0), Some(1.0));
        assert_eq!(decode_text_from_script(&script), text);
    }

    #[test]
    fn test_script_structure() {
        let script = build_sapi_script("测试", Some(1.5), Some(0.8));
        assert!(script.starts_with("$v = New-Object -ComObject SAPI.SpVoice; "));
        // (1.5 - 1.0) * 10 = 5；0.8 * 100 = 80
        assert!(script.contains("$v.Rate = 5; "));
        assert!(script.contains("$v.Volume = 80; "));
        assert!(script.contains("[void]$v.Speak($text);"));
    }

    #[test]
    fn test_script_without_rate_volume_uses_sapi_defaults() {
        // 未提供 rate/volume 时不设置（SAPI 默认 Rate=0 / Volume=100）
        let script = build_sapi_script("test", None, None);
        assert!(!script.contains("$v.Rate"));
        assert!(!script.contains("$v.Volume"));
    }

    #[test]
    fn test_map_rate_mapping() {
        // 倍率 1.0（正常语速）→ SAPI 0
        assert_eq!(map_rate(1.0), 0);
        // 半速 → -5，两倍速 → +10
        assert_eq!(map_rate(0.5), -5);
        assert_eq!(map_rate(2.0), 10);
        // 超出 SAPI 范围 [-10, 10] 时夹紧
        assert_eq!(map_rate(3.0), 10);
        assert_eq!(map_rate(0.0), -10);
        // 无穷/NaN 不 panic（NaN 饱和转换结果为 0）
        assert_eq!(map_rate(f32::INFINITY), 10);
        assert_eq!(map_rate(f32::NEG_INFINITY), -10);
        assert_eq!(map_rate(f32::NAN), 0);
    }

    #[test]
    fn test_map_volume_mapping() {
        // 0..1 → 0..100
        assert_eq!(map_volume(0.0), 0);
        assert_eq!(map_volume(0.5), 50);
        assert_eq!(map_volume(1.0), 100);
        // 越界夹紧
        assert_eq!(map_volume(1.5), 100);
        assert_eq!(map_volume(-0.3), 0);
        // 无穷/NaN 不 panic
        assert_eq!(map_volume(f32::INFINITY), 100);
        assert_eq!(map_volume(f32::NEG_INFINITY), 0);
        assert_eq!(map_volume(f32::NAN), 0);
    }
}
