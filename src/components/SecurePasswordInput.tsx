import React, { useState, useCallback } from 'react';
import { Shield, Copy, Check } from '@phosphor-icons/react';
import { useTranslation } from 'react-i18next';
import { copyTextToClipboard } from '@/utils/clipboardUtils';
import { ApiKeyField } from '@/features/settings/components/ApiKeyField';

interface SecurePasswordInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  disabled?: boolean;
  /** 是否为敏感键：开启后显示加密徽章，并在隐藏状态下禁用复制 */
  isSensitive?: boolean;
}

export const SecurePasswordInput: React.FC<SecurePasswordInputProps> = ({
  value,
  onChange,
  placeholder,
  className,
  disabled = false,
  isSensitive = false,
}) => {
  const { t } = useTranslation('common');
  const [showPassword, setShowPassword] = useState(false);
  const [copied, setCopied] = useState(false);

  const canReveal = value.trim().length > 0;
  const copyBlockedBySensitivity = isSensitive && !showPassword;
  const copyDisabled = disabled || copyBlockedBySensitivity || !value;

  // ★ WebView2 paste fix: reads pasted text synchronously from clipboardData,
  // bypasses React's patched value setter, and dispatches 'input' event.
  // setTimeout(0) is unreliable in Tauri WebView2 — the browser may not have
  // written the pasted text to input.value by the time the callback fires.
  const handlePaste = useCallback((e: React.ClipboardEvent<HTMLInputElement>) => {
    const pastedText = e.clipboardData?.getData('text/plain') ?? '';
    if (pastedText) {
      e.preventDefault();
      const input = e.currentTarget;
      const start = input.selectionStart ?? 0;
      const end = input.selectionEnd ?? 0;
      const newValue = input.value.slice(0, start) + pastedText + input.value.slice(end);

      const nativeSetter = Object.getOwnPropertyDescriptor(
        window.HTMLInputElement.prototype, 'value',
      )?.set;
      nativeSetter?.call(input, newValue);
      input.dispatchEvent(new Event('input', { bubbles: true }));

      if (newValue !== value) onChange(newValue);
    } else {
      // Fallback: clipboardData unavailable
      setTimeout(() => {
        const newValue = e.currentTarget.value;
        if (newValue !== value) onChange(newValue);
      }, 10);
    }
  }, [onChange, value]);

  const handleCopy = useCallback(async () => {
    if (copyDisabled) return;
    try {
      await copyTextToClipboard(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err: unknown) {
      console.warn('Failed to copy to clipboard:', err);
    }
  }, [value, copyDisabled]);

  const copyTitle = copyBlockedBySensitivity
    ? t('securePassword.showToCopy')
    : copied
      ? t('securePassword.copied')
      : t('actions.copy');

  const copyButton = value ? (
    // eslint-disable-next-line ds-components/no-native-button -- Input adornment needs exact height/edge control to align with the reveal toggle inside ApiKeyField shell.
    <button
      type="button"
      onClick={handleCopy}
      disabled={copyDisabled}
      aria-label={t('actions.copy')}
      title={copyTitle}
      className="api-key-field__action"
    >
      {copied ? (
        <Check className="api-key-field__icon text-green-600" />
      ) : (
        <Copy className="api-key-field__icon" />
      )}
    </button>
  ) : null;

  return (
    <div className={className}>
      <ApiKeyField
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onPaste={handlePaste}
        placeholder={placeholder}
        disabled={disabled}
        revealed={showPassword}
        canReveal={canReveal}
        onToggle={() => setShowPassword((prev) => !prev)}
        showLabel={t('securePassword.showPassword')}
        hideLabel={t('securePassword.hidePassword')}
        inputClassName="font-mono"
        autoComplete="new-password"
        inputMode={showPassword ? 'text' : undefined}
        extraActions={copyButton}
/>

      {isSensitive && (
        <div className="mt-1 flex items-center text-xs text-green-600">
          <Shield size={12} className="mr-1" />
          <span>{t('securePassword.encryptedInSecureArea')}</span>
        </div>
      )}
    </div>
  );
};
