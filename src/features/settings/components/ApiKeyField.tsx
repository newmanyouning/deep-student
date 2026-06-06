import * as React from 'react';
import { Eye, EyeSlash } from '@phosphor-icons/react';
import { cn } from '@/lib/utils';
import '../styles/api-key-field.css';

interface ApiKeyFieldProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'className' | 'type'> {
  revealed: boolean;
  canReveal: boolean;
  disabled?: boolean;
  showLabel: string;
  hideLabel: string;
  onToggle: () => void;
  inputClassName?: string;
  className?: string;
  /**
   * Optional slot for additional adornments (e.g. a copy button) rendered
   * inside the shell, to the left of the reveal toggle.
   * Use `.api-key-field__action` class on child buttons to match the
   * built-in toggle's styling.
   */
  extraActions?: React.ReactNode;
}

export const ApiKeyField = React.forwardRef<HTMLInputElement, ApiKeyFieldProps>(({
  revealed,
  canReveal,
  disabled,
  showLabel,
  hideLabel,
  onToggle,
  inputClassName,
  className,
  extraActions,
  onPaste: onPasteProp,
  onChange: onChangeProp,
  ...props
}, ref) => {
  const label = revealed ? hideLabel : showLabel;
  const inputType = canReveal && revealed ? 'text' : 'password';

  // ★ WebView2 paste fix: React controlled <input type="password"> sometimes
  // does NOT fire onChange when pasting (Ctrl+V / right-click paste) in Tauri
  // WebView2. setTimeout(0) is unreliable because the browser may not have
  // written the pasted text to input.value by the time it fires.
  //
  // Solution: read pasted text synchronously from clipboardData, then use
  // the native HTMLInputElement value setter (bypassing React's override)
  // followed by dispatching an 'input' event to trigger React's onChange.
  // Ref: https://github.com/facebook/react/issues/11488#issuecomment-347775628
  const handlePaste = React.useCallback((e: React.ClipboardEvent<HTMLInputElement>) => {
    onPasteProp?.(e);
    const input = e.currentTarget;
    const pastedText = e.clipboardData?.getData('text/plain') ?? '';

    if (pastedText) {
      // Prevent browser's default paste to avoid double-insert
      e.preventDefault();

      // Insert pasted text at cursor position (handles partial text selection)
      const start = input.selectionStart ?? 0;
      const end = input.selectionEnd ?? 0;
      const newValue = input.value.slice(0, start) + pastedText + input.value.slice(end);

      // Bypass React's patched value setter by using the native prototype setter
      const nativeSetter = Object.getOwnPropertyDescriptor(
        window.HTMLInputElement.prototype, 'value',
      )?.set;
      nativeSetter?.call(input, newValue);

      // Dispatch 'input' event so React's synthetic event system picks it up
      input.dispatchEvent(new Event('input', { bubbles: true }));

      // Also notify parent directly via onChange (belt-and-suspenders)
      if (onChangeProp) {
        onChangeProp({ target: input } as React.ChangeEvent<HTMLInputElement>);
      }
    } else {
      // Fallback: clipboardData unavailable (rare), rely on browser default +
      // setTimeout to pick up the value after the browser writes it
      setTimeout(() => {
        if (onChangeProp) {
          onChangeProp({ target: input } as React.ChangeEvent<HTMLInputElement>);
        }
      }, 10);
    }
  }, [onPasteProp, onChangeProp]);

  return (
    <div
      data-api-key-field
      className={cn(
        'api-key-field',
        disabled && 'api-key-field--disabled',
        className
      )}
    >
      <input
        ref={ref}
        type={inputType}
        disabled={disabled}
        className={cn(
          'api-key-field__input',
          inputClassName
        )}
        onPaste={handlePaste}
        onInput={onChangeProp as unknown as React.FormEventHandler<HTMLInputElement>}
        onChange={onChangeProp}
        {...props}
      />
      {extraActions}
      {canReveal && (
        // eslint-disable-next-line ds-components/no-native-button -- Input adornment needs exact height/edge control instead of shared button primitive sizing.
        <button
          type="button"
          onClick={onToggle}
          disabled={disabled}
          aria-label={label}
          aria-pressed={revealed}
          title={label}
          className="api-key-field__action api-key-field__toggle"
        >
          {revealed ? <Eye className="api-key-field__icon" /> : <EyeSlash className="api-key-field__icon" />}
        </button>
      )}
    </div>
  );
});

ApiKeyField.displayName = 'ApiKeyField';
