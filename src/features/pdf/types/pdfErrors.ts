/**
 * pdfErrors - PDF 加载错误类型系统
 *
 * 用法:
 *   const err = classifyPdfError(someError, { found: result?.found, isBase64: true });
 *   if (err.actionable) { showReImportSuggestion(); }
 */

/**
 * PDF 错误类型枚举
 */
export enum PdfErrorType {
  /** 文件内容未找到（数据库查询返回空） */
  NotFound = 'NotFound',
  /** Base64 解析/格式转换/解码失败 */
  Corrupted = 'Corrupted',
  /** 文件格式非法或解析失败 */
  FormatError = 'FormatError',
  /** 加载超时 */
  LoadTimeout = 'LoadTimeout',
  /** 处理过程中的内部错误 */
  ProcessingError = 'ProcessingError',
  /** 权限不足（403/forbidden） */
  PermissionDenied = 'PermissionDenied',
  /** 未分类的未知错误 */
  Unknown = 'Unknown',
}

/**
 * PDF 加载错误
 */
export interface PdfLoadError {
  /** 错误分类 */
  type: PdfErrorType;
  /** 人类可读的错误消息 */
  message: string;
  /** 可选的详细技术信息 */
  detail?: string;
  /** 是否可操作（用户能否采取行动解决） */
  actionable: boolean;
}

/**
 * 分类 PDF 加载错误
 *
 * 根据错误消息和上下文信息，将原始 error 分类为结构化的 PdfLoadError。
 *
 * @param error 原始错误（字符串、Error 或任何未知类型）
 * @param context 可选的上下文信息
 * @param context.found 数据库查询是否返回了结果
 * @param context.isBase64 是否是在 base64 处理阶段发生的错误
 */
export function classifyPdfError(
  error: unknown,
  context?: { found?: boolean; isBase64?: boolean },
): PdfLoadError {
  const rawMessage = extractMessage(error);

  // 1. 根据 context.found 判断 NotFound
  if (context?.found === false) {
    return {
      type: PdfErrorType.NotFound,
      message: rawMessage || 'PDF file content not found',
      detail: 'The database returned no content for this attachment. The file may not have been imported correctly.',
      actionable: true,
    };
  }

  // 2. 根据 context.isBase64 判断 Corrupted（在 base64 阶段明确出错）
  if (context?.isBase64) {
    return {
      type: PdfErrorType.Corrupted,
      message: rawMessage || 'Failed to decode PDF content from base64',
      detail: 'The base64-encoded content could not be decoded. The file data may be corrupted or truncated.',
      actionable: true,
    };
  }

  const lower = rawMessage.toLowerCase();

  // 3. PermissionDenied — 403 / forbidden / permission
  if (
    lower.includes('403') ||
    lower.includes('forbidden') ||
    lower.includes('permission')
  ) {
    return {
      type: PdfErrorType.PermissionDenied,
      message: rawMessage,
      detail: 'Access to the PDF file was denied. It may require authentication or valid credentials.',
      actionable: false,
    };
  }

  // 4. LoadTimeout — timeout
  if (lower.includes('timeout')) {
    return {
      type: PdfErrorType.LoadTimeout,
      message: rawMessage,
      detail: 'The PDF load request timed out. The file may be too large or the server may be slow.',
      actionable: true,
    };
  }

  // 5. Corrupted — base64 / conversion / decode
  if (
    lower.includes('base64') ||
    lower.includes('conversion') ||
    lower.includes('decode') ||
    lower.includes('convert')
  ) {
    return {
      type: PdfErrorType.Corrupted,
      message: rawMessage,
      detail: 'The PDF content could not be decoded or converted. The file may be corrupted.',
      actionable: true,
    };
  }

  // 6. NotFound — content not found
  if (lower.includes('content not found')) {
    return {
      type: PdfErrorType.NotFound,
      message: rawMessage,
      detail: 'The PDF file content was not found in the system.',
      actionable: true,
    };
  }

  // 7. FormatError — parse / format / invalid
  if (
    lower.includes('parse') ||
    lower.includes('format') ||
    lower.includes('invalid')
  ) {
    return {
      type: PdfErrorType.FormatError,
      message: rawMessage,
      detail: 'The file does not appear to be a valid PDF or its format is not supported.',
      actionable: false,
    };
  }

  // 8. fallback: Unknown
  return {
    type: PdfErrorType.Unknown,
    message: rawMessage || 'An unknown error occurred while loading the PDF',
    actionable: false,
  };
}

/**
 * 从未知类型 error 中提取字符串消息
 */
function extractMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (error && typeof error === 'object') {
    // 尝试常见字段
    const obj = error as Record<string, unknown>;
    if (typeof obj.message === 'string') return obj.message;
    if (typeof obj.error === 'string') return obj.error;
    if (typeof obj.detail === 'string') return obj.detail;
  }
  return String(error);
}
